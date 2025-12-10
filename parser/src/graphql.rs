use std::fmt::Debug;

use graphql_client::{GraphQLQuery, Response};

pub const MENSA_GRAPHQL_URL: &str = "https://mensa.k8s.incloud.de/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/menuitems.graphql",
    response_derives = "Debug",
    response_derives = "serde::Serialize"
)]
pub struct MenuItems;

pub async fn send_query<T: GraphQLQuery>(
    variables: T::Variables,
) -> anyhow::Result<T::ResponseData> {
    let request_body = T::build_query(variables);

    let res = reqwest::Client::new()
        .post(MENSA_GRAPHQL_URL)
        .header(reqwest::header::AUTHORIZATION, "openmensa-parser")
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?;

    let response: Response<T::ResponseData> = res.json().await?;

    match response.data {
        Some(items) => Ok(items),
        None => match response.errors {
            Some(errors) => {
                let errors_string = errors
                    .iter()
                    .map(|e| format!("{:#?}", e))
                    .collect::<Vec<String>>()
                    .join(", ");
                Err(anyhow::anyhow!(errors_string))
            }
            None => Err(anyhow::anyhow!(
                "graphql response with neither data or errors",
            )),
        },
    }
}

pub fn allergic_descriptive(allergic: &str) -> &str {
    match allergic {
        "A" => "Glutenhaltiges Getreide",
        "A1" => "Weizen",
        "A2" => "Dinkel",
        "A3" => "Roggen",
        "A4" => "Gerste",
        "A5" => "Hafer",

        "B" => "Krebstiere und Krebstiererzeugnisse",
        "C" => "Eier und Eiererzeugnisse",
        "D" => "Fisch und Fischerzeugnisse",
        "E" => "Erdnüsse und Erdnusserzeugnisse",
        "F" => "Soja und Sojaerzeugnisse",
        "G" => "Milch und Milcherzeugnisse",

        "H" => "Schalenfrüchte",
        "H1" => "Mandeln",
        "H2" => "Haselnüsse",
        "H3" => "Walnüsse",
        "H4" => "Cashewnüsse",
        "H5" => "Pekannüsse",
        "H6" => "Paranüsse",
        "H7" => "Pistazien",
        "H8" => "Macadamianüsse",

        "I" => "Sellerie und Sellerieerzeugnisse",
        "J" => "Senf und Senferzeugnisse",
        "K" => "Sesamsamen und Sesamsamenerzeugnisse",
        "L" => "Schwefeloxid und Sulfite",
        "M" => "Lupine und Lupinenerzeugnisse",
        "N" => "Weichtiere (Mollusken)",

        _ => {
            tracing::warn!("encountered unknown allergic: {}", allergic);
            ""
        }
    }
}

pub fn additive_descriptive(additive: &str) -> &str {
    match additive {
        "1" => "Lebensmittelfarbe",
        "2" => "Konservierungsstoffe",
        "3" => "Antioxidationsmittel",
        "4" => "Geschmacksverstärker",
        "5" => "Geschwefelt",
        "6" => "Geschwärzt",
        "7" => "Gewachst",
        "8" => "Phosphat",
        "9" => "Süßungsmittel",
        "10" => "Phenylalaninquelle",

        _ => {
            tracing::warn!("encountered unknown additive descriptive");
            ""
        }
    }
}

pub fn type_descriptive(type_: &menu_items::DishType) -> Option<String> {
    use menu_items::DishType;
    match type_ {
        DishType::VEGAN => Some("Vegan".into()),
        DishType::MEATLESS => Some("Vegetarisch".into()),
        DishType::PORK => Some("Schweinefleisch".into()),
        DishType::POULTRY => Some("Geflügel".into()),
        DishType::FISH => Some("Fisch".into()),
        DishType::BEEF => Some("Rind".into()),
        _ => None,
    }
}
