use std::str::FromStr;

use clap::Parser;
use openmensa_parser_darmstadt::{openmensa, parser};
use tokio::io::AsyncWriteExt;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone)]
struct FeedInput {
    pub canteen_id: String,
    pub name: String,
    pub priority: Option<i32>,
    pub url: String,
    pub hour: String,
    pub minute: Option<String>,
    pub day_of_week: Option<String>,
    pub day_of_month: Option<String>,
    pub month: Option<String>,
    pub retry: Option<String>,
}

impl FromStr for FeedInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(';').collect();
        if parts.len() < 5 {
            anyhow::bail!("Feed format: canteen;name;priority;url;hour[;minute[;dayOfWeek[;dayOfMonth[;month[;retry]]]]]");
        }

        Ok(Self {
            canteen_id: parts[0].to_string(),
            name: parts[1].to_string(),
            priority: if parts[2].is_empty() {
                None
            } else {
                Some(parts[2].parse()?)
            },
            url: parts[3].to_string(),
            hour: parts[4].to_string(),
            minute: parts.get(5).filter(|&s| !s.is_empty()).map(|s| s.to_string()),
            day_of_week: parts.get(6).filter(|&s| !s.is_empty()).map(|s| s.to_string()),
            day_of_month: parts.get(7).filter(|&s| !s.is_empty()).map(|s| s.to_string()),
            month: parts.get(8).filter(|&s| !s.is_empty()).map(|s| s.to_string()),
            retry: parts.get(9).filter(|&s| !s.is_empty()).map(|s| s.to_string()),
        })
    }
}

impl Into<openmensa::Feed> for FeedInput {
    fn into(self) -> openmensa::Feed {
        openmensa::Feed {
            name: self.name,
            priority: self.priority,
            url: self.url,
            source: None,
            schedule: Some(openmensa::Schedule {
                hour: self.hour,
                minute: self.minute,
                day_of_week: self.day_of_week,
                day_of_month: self.day_of_month,
                month: self.month,
                retry: self.retry,
            }),
        }
    }
}
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, required = true, num_args = 1..)]
    canteen: Vec<String>,
    #[arg(short, long)]
    from: Option<chrono::NaiveDate>,
    #[arg(short, long)]
    to: Option<chrono::NaiveDate>,
    #[arg(short, long, default_value = "./out")]
    out: std::path::PathBuf,
    #[arg(
        long, 
        required = false, 
        num_args = 1..,
        help = "Feed format: CANTEEN;NAME;PRIORITY;URL;HOUR[;MINUTE[;DAY_OF_WEEK[;DAY_OF_MONTH[;MONTH[;RETRY]]]]]. Examples:\n  --feed \"1;full;1;https://openmensa.example.com/full/1.xml;4;*;*;*;60 5 1440\""
    )]
    feed: Vec<FeedInput>,
}

async fn fetch_and_write_canteen_data(
    canteen_id: String,
    from: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
    out: &std::path::PathBuf,
    feeds: Option<Vec<openmensa::Feed>>,
) -> anyhow::Result<()> {
    let mut data = parser::fetch_openmensa_for_range(canteen_id.clone(), from, to).await?;

    if let Some(feeds) = feeds {
        data.canteen.feeds.extend(feeds);
    }

    let mut file = tokio::fs::File::create(out).await?;
    file.write_all(&data.serialize_to_string()?.as_bytes())
        .await?;

    tracing::debug!(
        "wrote data for canteen \"{}\" to {}",
        canteen_id,
        out.to_string_lossy()
    );
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let args = Args::parse();
    tracing::debug!("args: {:?}", args);

    if let Err(e) = tokio::fs::create_dir_all(&args.out).await {
        tracing::error!(
            "failed to create \"{}\": {:?}",
            args.out.to_string_lossy(),
            e
        );
        panic!("failed to create out dir");
    }

    let mut set = tokio::task::JoinSet::new();

    let feed_map: std::collections::HashMap<String, Vec<openmensa::Feed>> = args.feed.into_iter()
    .map(|f| f.into()) // Convert to Feed
    .fold(std::collections::HashMap::new(), |mut acc, feed_input: FeedInput| {
        let feed = openmensa::Feed {
            name: feed_input.name,
            priority: feed_input.priority,
            url: feed_input.url,
            source: None,
            schedule: Some(openmensa::Schedule {
                hour: feed_input.hour,
                minute: feed_input.minute,
                day_of_week: feed_input.day_of_week,
                day_of_month: feed_input.day_of_month,
                month: feed_input.month,
                retry: feed_input.retry,
            }),
        };
        acc.entry(feed_input.canteen_id).or_insert_with(Vec::new).push(feed);
        acc
    });

    for canteen_id in args.canteen {
        let filename = args.out.join(format!("{canteen_id}.xml"));
        let from = args.from;
        let to = args.to;
        let feeds = feed_map.get(&canteen_id).map(|v| v.clone());

        set.spawn(async move {
            if let Err(e) = fetch_and_write_canteen_data(canteen_id, from, to, &filename, feeds).await {
                tracing::error!("failed to fetch/write data: {:?}", e);
            }
        });
    }

    while let Some(res) = set.join_next().await {
        if let Err(e) = res {
            panic!("task panicked: {:?}", e);
        }
    }
}
