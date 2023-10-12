use futures_util::future::join_all;
use std::error;
use std::fs::File;
use std::io::Cursor;
use std::num::ParseIntError;
use std::path::PathBuf;
pub mod errors;

mod zip_wrapper;

use errors::*;
use tokio::fs::{ self, create_dir_all, remove_dir_all };
use tokio::task::JoinSet;
use tracing::{ info, warn };

const ROBLOX_DIRS: [[&str; 2]; 25] = [
    ["RobloxApp.zip", ""],
    ["shaders.zip", "shaders/"],
    ["ssl.zip", "ssl/"],
    ["Libraries.zip", ""],
    ["WebView2.zip", ""],
    ["content-translations.zip", "translations/"],
    ["content-luapackages.zip", "content/luapackages"],
    ["WebView2RuntimeInstaller.zip", "WebView2RuntimeInstaller"],
    ["NPRobloxProxy.zip", ""],
    ["content-avatar.zip", "content/avatar"],
    ["content-configs.zip", "content/configs"],
    ["content-fonts.zip", "content/fonts"],
    ["content-sky.zip", "content/sky"],
    ["content-sounds.zip", "content/sounds"],
    ["content-textures2.zip", "content/textures"],
    ["content-models.zip", "content/models"],
    ["content-textures3.zip", "PlatformContent/pc/textures"],
    ["content-terrain.zip", "PlatformContent/pc/terrain"],
    ["content-platform-fonts.zip", "PlatformContent/pc/fonts"],
    ["extracontent-luapackages.zip", "ExtraContent/LuaPackages"],
    ["extracontent-translations.zip", "ExtraContent/translations"],
    ["extracontent-models.zip", "ExtraContent/models"],
    ["extracontent-textures.zip", "ExtraContent/textures"],
    ["extracontent-places.zip", "ExtraContent/places"],
    ["extracontent-scripts.zip", "ExtraContent/scripts"],
];

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Downloader {
    pub cdn: String,
    pub version_hash: String,

    pub roblox_packages: Vec<RbxPackage>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RbxPackage {
    filename: String,
    md5: String,

    compressed_size: u64,
    decompressed_size: u64,
}

fn get_file_dir(file: &str) -> PathBuf {
    for [file1, file2] in ROBLOX_DIRS {
        if file == file1 {
            return PathBuf::from(format!("./{}", file2));
        }
    }
    tracing::warn!("{} is exiting at root", file);
    return PathBuf::from("./");
}

async fn extract_async(
    bytes: Cursor<Vec<u8>>,
    dir: PathBuf
) -> std::result::Result<(), ZipExtractionError> {
    println!("{:?}", dir.display());
    fs::create_dir_all(&dir).await.or(Err(ZipExtractionError))?;
    zip_extract::extract(bytes, &dir, false).or(Err(ZipExtractionError))
}

impl Downloader {
    pub async fn populate(&mut self) -> Result<()> {
        let manifest_url = format!("{}/{}-rbxPkgManifest.txt", self.cdn, self.version_hash);

        let as_text = reqwest::get(manifest_url).await?.text().await?;

        let mut lines: Vec<&str> = as_text.lines().collect();

        // There is a V0 at the start of each roblox manifest
        lines.remove(0);

        let chunks: Vec<&[&str]> = lines.chunks(4).collect();

        for chunk in chunks {
            self.roblox_packages.push(RbxPackage::from(chunk)?);
        }

        Ok(())
    }

    fn get_url(&self) -> String {
        format!("{}/{}", self.cdn, self.version_hash)
    }

    pub async fn download_and_package(
        &self,
        download_location: PathBuf,
        package_location: PathBuf
    ) -> crate::common::Result<()> {
        let creation = create_dir_all(&package_location);
        let download_task = self.download(download_location.clone());
        let walkdir = walkdir::WalkDir::new(download_location.clone());

        let it = walkdir.into_iter();
        let location: crate::common::Result<&str> = download_location
            .to_str()
            .ok_or(FileCreationError.into());

        let pkg_location = package_location.join(format!("./{}.zip", &self.version_hash));

        if pkg_location.exists() {
            return Err(FileAlreadyExistError.into());
        }

        download_task.await?;
        creation.await?;
        let file = File::create(package_location.join(format!("./{}.zip", &self.version_hash)))?;

        zip_wrapper::zip_dir(
            &mut it.filter_map(|e| e.ok()),
            location?,
            file,
            zip::CompressionMethod::Bzip2
        )?;

        remove_dir_all(download_location).await?;

        Ok(())
    }

    pub async fn download(&self, download_location: PathBuf) -> Result<()> {
        info!("Starting download from {}", &self.get_url());
        create_dir_all(&download_location).await?;

        let mut promises = vec![];

        for pkg in &self.roblox_packages {
            let url = self.get_url();
            promises.push(pkg.download(url));
        }

        let res = join_all(promises).await;

        let mut extraction_jobs = JoinSet::new();

        for (index, result) in res.iter().enumerate() {
            let bytes = result.as_ref().unwrap();
            let hash = format!("{:x}", md5::compute(bytes));
            let pkg = &self.roblox_packages[index];

            let is_valid = hash == pkg.md5;

            if !is_valid {
                tracing::error!(
                    "{} Is not valid aborting (Expected {} Got {})",
                    pkg.filename,
                    pkg.md5,
                    hash
                );
                return Err(InvalidMd5Hash.into());
            }

            info!("{} Validated  {}", pkg.filename, is_valid);

            if pkg.filename.ends_with(".exe") || pkg.compressed_size == pkg.decompressed_size {
                let path = &download_location.join(format!("./{}", pkg.filename));
                File::create(path)?;
            } else {
                extraction_jobs.spawn(
                    extract_async(
                        Cursor::new(bytes.clone()),
                        download_location.join(get_file_dir(&pkg.filename))
                    )
                );
            }
            continue;
        }

        let req = reqwest
            ::get(format!("{}-rbxManifest.txt", &self.get_url())).await?
            .bytes().await?;

        let write = fs::write(download_location.join("./rbxManifest.txt"), req);

        while let Some(extraction) = extraction_jobs.join_next().await {
            info!("Job finished checking status");

            match extraction {
                Ok(_) => {
                    info!("No issues when extracting");
                }
                Err(_) => {
                    return Err(ZipExtractionError.into());
                }
            }
        }
        write.await?;
        Ok(())
    }
}

impl RbxPackage {
    pub async fn download(&self, version_plus_cdn: String) -> Result<Vec<u8>> {
        info!("Downloading file {}", self.filename);
        let client = reqwest::Client::builder().build()?;

        let request = client.get(format!("{}-{}", version_plus_cdn, self.filename)).send().await?;

        // Dumb reason to throw for
        let total_size_result: Result<u64> = request
            .content_length()
            .ok_or_else(|| NoTotalSize.into());

        let total_size = total_size_result.unwrap_or(0);

        if total_size != self.compressed_size {
            //return Err(BadTotalSize.into());
            warn!("Bad total size file may not be original");
        }

        let bytes = request.bytes().await?.to_vec();

        info!("Downloaded file {}", self.filename);

        Ok(bytes)
    }

    pub fn from(chunks: &[&str]) -> std::result::Result<RbxPackage, ParseIntError> {
        let file_name = chunks[0].to_string();
        let file_hash = chunks[1].to_string();

        let compressed_size: u64 = chunks[2].parse()?;
        let decompressed_size: u64 = chunks[3].parse()?;

        Ok(RbxPackage {
            filename: file_name,
            md5: file_hash,
            compressed_size: compressed_size,
            decompressed_size: decompressed_size,
        })
    }
}
