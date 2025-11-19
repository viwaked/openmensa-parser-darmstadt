use std::collections::HashMap;

use crate::{
    graphql::{
        self, MenuItems, additive_descriptive, allergic_descriptive, menu_items, type_descriptive,
    },
    openmensa,
};

pub async fn fetch_openmensa_for_range(
    canteen_id: String,
    from_date: Option<chrono::NaiveDate>,
    to_date: Option<chrono::NaiveDate>,
) -> anyhow::Result<openmensa::OpenMensa> {
    let from_date = from_date.map(|v| v.to_string());
    let to_date = to_date.map(|v| v.to_string());

    let menu_items = graphql::send_query::<MenuItems>(menu_items::Variables {
        canteen_id,
        lang: menu_items::Language::DE,
        min_date: from_date,
        max_date: to_date,
    })
    .await?
    .menu_items;

    let mut grouped_items: HashMap<chrono::NaiveDate, Vec<menu_items::MenuItemsMenuItemsDish>> =
        HashMap::new();
    for item in menu_items {
        let date = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(item.date)
            .ok_or(anyhow::anyhow!("failed to decode item date"))?
            .date_naive();

        grouped_items
            .entry(date)
            .or_insert_with(Vec::new)
            .push(item.dish);
    }

    let mut grouped_items = grouped_items.iter().collect::<Vec<_>>();
    grouped_items.sort_by_key(|(k, _)| *k);

    let notes = |dish: &menu_items::MenuItemsMenuItemsDish| -> Vec<String> {
        let mut notes = Vec::new();

        if let Some(type_) = type_descriptive(&dish.type_) {
            notes.push(type_);
        }

        let allergics = dish.allergics.iter().map(|a| {
            let mut descriptive = allergic_descriptive(a).to_string();

            if let Some(specifics) = &dish.specific_allergics {
                let specifics: Vec<&str> = specifics
                    .iter()
                    .filter_map(|v| match v.starts_with(a) {
                        true => Some(allergic_descriptive(v)),
                        false => None,
                    })
                    .collect();

                if !specifics.is_empty() {
                    descriptive += &format!(" ({})", specifics.join(", "));
                }
            }

            descriptive
        });
        notes.extend(allergics);

        notes.extend(
            dish.additionals
                .iter()
                .map(|a| additive_descriptive(a).to_string()),
        );

        notes
    };

    let openmensa = openmensa::OpenMensa {
        version: "2.1".into(),
        parser_version: option_env!("CARGO_PKG_VERSION").map(|v| v.into()),
        canteen: openmensa::Canteen {
            days: grouped_items
                .iter()
                .map(|(date, dishes)| openmensa::Day {
                    date: date.to_string(),
                    content: openmensa::DayContent::Open {
                        category: vec![openmensa::Category {
                            name: "Mensa".into(),
                            meal: dishes
                                .iter()
                                .map(|dish| openmensa::Meal {
                                    name: dish.name.clone(),
                                    notes: notes(&dish),
                                    prices: vec![
                                        openmensa::Price {
                                            role: openmensa::PriceRole::Student,
                                            value: dish.student_price as f32,
                                        },
                                        openmensa::Price {
                                            role: openmensa::PriceRole::Other,
                                            value: dish.guest_price as f32,
                                        },
                                    ],
                                })
                                .collect(),
                        }],
                    },
                })
                .collect::<Vec<_>>(),
            ..Default::default()
        },
    };

    Ok(openmensa)
}
