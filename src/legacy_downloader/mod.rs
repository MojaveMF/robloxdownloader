pub enum Version {
    GAMETEST2,
}

impl Version {
    pub fn to_string(&self) -> String {
        match self {
            Version::GAMETEST2 => "gametest2".to_string(),
        }
    }

    /* 
    pub fn _from_string<T: ToString>(as_ref: T) -> Version {
        let str = as_ref.to_string();
        todo!()
    }
    */
    pub fn to_url(&self) -> String {
        format!("http://setup.{}.robloxlabs.com", self.to_string())
    }
}

use crate::common::Downloader;

#[allow(non_snake_case)]
pub async fn Latest() -> crate::common::Result<Downloader> {
    let result: String = reqwest
        ::get(format!("{}/version", Version::GAMETEST2.to_url())).await?
        .text().await?;

    let mut downloader = Downloader {
        cdn: Version::GAMETEST2.to_url(),
        version_hash: result,
        roblox_packages: vec![],
    };

    downloader.populate().await?;

    Ok(downloader)
}

#[allow(non_snake_case)]
pub fn From<T: ToString>(as_ref: T) -> Downloader {
    Downloader {
        cdn: Version::GAMETEST2.to_url(),
        version_hash: as_ref.to_string(),
        roblox_packages: vec![],
    }
}
