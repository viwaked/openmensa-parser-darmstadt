use clap::Parser;
use openmensa_parser_darmstadt::parser;
use tokio::io::AsyncWriteExt;
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, required = true)]
    canteen: Vec<String>,
    #[arg(short, long)]
    from: Option<chrono::NaiveDate>,
    #[arg(short, long)]
    to: Option<chrono::NaiveDate>,
    #[arg(short, long, default_value = "./out")]
    out: std::path::PathBuf,
}

async fn fetch_and_write_canteen_data(
    canteen_id: String,
    from: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
    out: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let data = parser::fetch_openmensa_for_range(canteen_id.clone(), from, to).await?;

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

    for canteen_id in args.canteen {
        let filename = args.out.join(format!("{canteen_id}.xml"));
        let from = args.from;
        let to = args.to;

        set.spawn(async move {
            if let Err(e) = fetch_and_write_canteen_data(canteen_id, from, to, &filename).await {
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
