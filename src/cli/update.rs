use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use dialoguer::Confirm;
use semver::Version;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const GITHUB_API_URL: &str = "https://api.github.com/repos/dvaji/infera/releases/latest";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Args)]
pub struct UpdateCommands {
    #[command(subcommand)]
    pub command: UpdateSubcommands,
}

#[derive(Subcommand)]
pub enum UpdateSubcommands {
    /// Check for updates (without installing)
    Check,
    /// Update infs to the latest version
    Update {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

pub async fn handle_update_command(command: UpdateCommands, json: bool) -> Result<()> {
    match command.command {
        UpdateSubcommands::Check => check_for_update(json).await,
        UpdateSubcommands::Update { yes } => self_update(yes, json).await,
    }
}

async fn check_for_update(json: bool) -> Result<()> {
    let release = fetch_latest_release().await?;
    let latest_version = parse_version(&release.tag_name)?;
    let current_version = Version::parse(VERSION)?;

    if json {
        let output = serde_json::json!({
            "current_version": VERSION,
            "latest_version": latest_version.to_string(),
            "update_available": latest_version > current_version,
            "release_url": format!("https://github.com/dvaji/infera/releases/tag/{}", release.tag_name)
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Current version: {}", VERSION);
        println!("Latest version:  {}", latest_version);

        if latest_version > current_version {
            println!("\nUpdate available! Run `infs self update` to update.");
        } else {
            println!("\nYou're on the latest version.");
        }
    }

    Ok(())
}

async fn self_update(skip_confirm: bool, json: bool) -> Result<()> {
    let release = fetch_latest_release().await?;
    let latest_version = parse_version(&release.tag_name)?;
    let current_version = Version::parse(VERSION)?;

    if latest_version <= current_version {
        if json {
            let output = serde_json::json!({
                "status": "up_to_date",
                "current_version": VERSION,
                "latest_version": latest_version.to_string()
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Already on the latest version ({})", VERSION);
        }
        return Ok(());
    }

    let asset_name = get_asset_name_for_platform()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .context(format!(
            "No release asset found for your platform: {}",
            asset_name
        ))?;

    if json {
        let output = serde_json::json!({
            "status": "update_available",
            "current_version": VERSION,
            "latest_version": latest_version.to_string(),
            "download_url": asset.browser_download_url
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("Update available: {} -> {}", VERSION, latest_version);
    println!("Download URL: {}", asset.browser_download_url);

    if !skip_confirm
        && !Confirm::new()
            .with_prompt("Do you want to update?")
            .default(true)
            .interact()?
    {
        println!("Update cancelled.");
        return Ok(());
    }

    println!("Downloading latest version...");
    let new_binary = download_binary(&asset.browser_download_url).await?;

    println!("Installing update...");
    replace_current_binary(&new_binary)?;

    println!("Successfully updated to {}!", latest_version);
    Ok(())
}

async fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::builder()
        .user_agent(format!("infs/{}", VERSION))
        .build()?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .context("Failed to fetch release info from GitHub")?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API returned status: {}", response.status());
    }

    let release: GitHubRelease = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    Ok(release)
}

fn parse_version(tag: &str) -> Result<Version> {
    let version_str = tag.trim_start_matches('v').trim_start_matches("infs-");
    Version::parse(version_str).context(format!("Failed to parse version from tag: {}", tag))
}

fn get_asset_name_for_platform() -> Result<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => Ok("infs-linux-x86_64".to_string()),
        ("linux", "aarch64") => Ok("infs-linux-aarch64".to_string()),
        ("macos", "aarch64") => Ok("infs-macos-aarch64".to_string()),
        ("windows", "x86_64") => Ok("infs-windows-x86_64.exe".to_string()),
        ("macos", "x86_64") => {
            anyhow::bail!(
                "Intel macOS is not supported. Only Apple Silicon (aarch64) builds are available."
            )
        }
        ("windows", "aarch64") => {
            anyhow::bail!("Windows ARM64 is not supported. Only x86_64 builds are available.")
        }
        _ => anyhow::bail!("Unsupported platform: {}-{}", os, arch),
    }
}

async fn download_binary(url: &str) -> Result<PathBuf> {
    let client = reqwest::Client::builder()
        .user_agent(format!("infs/{}", VERSION))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download binary")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read download response")?;

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let extension = if env::consts::OS == "windows" {
        "exe"
    } else {
        "bin"
    };
    let temp_file = temp_dir.path().join(format!("infs.{}", extension));

    fs::write(&temp_file, &bytes).context("Failed to write downloaded binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permissions")?;
    }

    Ok(temp_file)
}

fn replace_current_binary(new_binary: &PathBuf) -> Result<()> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    #[cfg(windows)]
    {
        let backup_path = current_exe.with_extension("exe.old");
        if backup_path.exists() {
            fs::remove_file(&backup_path).ok();
        }
        fs::rename(&current_exe, &backup_path).context("Failed to backup current binary")?;
        fs::copy(new_binary, &current_exe).context("Failed to install new binary")?;
    }

    #[cfg(unix)]
    {
        fs::copy(new_binary, &current_exe).context("Failed to replace binary")?;
    }

    Ok(())
}
