use std::path::PathBuf;

use common::{ Downloader, errors::BadCliUse };

#[allow(dead_code)]
mod common;
mod legacy_downloader;
mod modern_downloader;

#[tokio::main]
async fn main() -> common::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    std::process::exit(real_main().await?);

    //Ok(());
}

async fn real_main() -> common::Result<i32> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 1 {
        println!("Usage <gametest2,live> <version-xxxxxxx (can be empty to use latest)>");
        return Ok(1);
    }

    let executable = &*args[0];

    let mut downloader: Downloader;

    match args.len() {
        1 => {
            downloader = modern_downloader::Latest().await?;
        }
        2 => {
            let legacy_or_modern = &*args[1];

            match legacy_or_modern {
                "gametest2" => {
                    downloader = legacy_downloader::Latest().await?;
                }
                "live" => {
                    downloader = modern_downloader::Latest().await?;
                }
                _ => {
                    return Err(common::errors::BadCliUse.into());
                }
            }
        }
        // 3 or more
        _ => {
            let legacy_or_modern = &*args[1];
            let version_hash = &*args[2];

            if !version_hash.starts_with("version-") {
                return Err(BadCliUse.into());
            }

            match legacy_or_modern {
                "gametest2" => {
                    downloader = legacy_downloader::From(version_hash);
                    downloader.populate().await;
                }
                "live" => {
                    downloader = modern_downloader::From(version_hash).await?;
                }
                _ => {
                    return Err(common::errors::BadCliUse.into());
                }
            }
        }
    }

    let res = downloader.download_and_package(
        PathBuf::from("./temp_rbx_downloader"),
        PathBuf::from(format!("./"))
    ).await;

    match res {
        Ok(_) => {}
        Err(_) => {
            res?;
        }
    }

    Ok(0)
}
