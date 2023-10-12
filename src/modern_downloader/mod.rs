use std::{ io::{ Error, ErrorKind }, vec };
use tokio::{ task::JoinSet, time::Instant };
use tracing::info;

pub mod errors;

use crate::common::Downloader;

use self::errors::*;

#[allow(dead_code)]
const MODERN_CDN: [&str; 6] = [
    "https://setup.rbxcdn.com",
    "https://s3.amazonaws.com/setup.roblox.com",
    "https://setup-ak.rbxcdn.com",
    "https://setup-hw.rbxcdn.com",
    "https://setup-cfly.rbxcdn.com",
    "https://roblox-setup.cachefly.net",
];
#[allow(dead_code)]
async fn time_request(url: String) -> Result<(f32, String), Error> {
    info!("Testing cdn {}", url);
    let timer = Instant::now();
    let req = reqwest
        ::get(format!("{}/version", &url)).await
        .or(Err(Error::new(ErrorKind::InvalidData, "Bad request")))?;

    match req.status() {
        reqwest::StatusCode::OK => {}
        _ => {
            return Err(Error::new(ErrorKind::InvalidData, "Bad request"));
        }
    }

    let time_elapsed = timer.elapsed().as_secs_f32();
    info!("Cdn elapsed time {}", time_elapsed);
    Ok((time_elapsed, url))
}

#[allow(non_snake_case)]
pub async fn Latest() -> crate::common::Result<Downloader> {
    let res: crate::common::Result<String> = find_fastest().await.ok_or(NoCdnError.into());
    let cdn = res?;

    let latest_version = reqwest::get(format!("{}/version", cdn)).await?.text().await?;

    let mut downloader = Downloader {
        cdn: cdn,
        version_hash: latest_version,
        roblox_packages: vec![],
    };

    downloader.populate().await?;

    Ok(downloader)
}

#[allow(dead_code)]
pub async fn find_fastest() -> Option<String> {
    let mut joinset: JoinSet<Result<(f32, String), Error>> = JoinSet::new();

    for cdn in MODERN_CDN {
        joinset.spawn(time_request(cdn.to_string()));
    }

    let mut fastest: Option<(f32, String)> = None;

    while let Some(val) = joinset.join_next().await {
        if val.is_err() {
            continue;
        }
        let nested_val = val.unwrap();
        if nested_val.is_err() {
            continue;
        }
        let (time, string) = nested_val.unwrap();

        if fastest.is_none() {
            fastest = Some((time, string));
            continue;
        }
        if fastest.is_some() {
            let (fastest_time, _) = fastest.clone().unwrap();
            if time < fastest_time {
                fastest = Some((time, string));
            }
        }
        continue;
    }
    let val = fastest.unwrap();
    Some(val.1)
}

#[allow(non_snake_case)]
pub async fn From<T: ToString>(as_ref: T) -> crate::common::Result<Downloader> {
    let cdn: crate::common::Result<String> = find_fastest().await.ok_or(NoCdnError.into());
    Ok(Downloader {
        cdn: cdn?,
        version_hash: as_ref.to_string(),
        roblox_packages: vec![],
    })
}
