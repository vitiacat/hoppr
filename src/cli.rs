use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Modrinth CLI tool")]
pub struct Cli {
    /// Path to the manifest file
    #[arg(short, long, default_value = "manifest.csv")]
    pub file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new manifest file (manifest.csv)
    Init(InitArgs),

    /// Add one or more projects to the manifest
    Add(AddArgs),

    /// Remove a project from the manifest
    Remove(RemoveArgs),

    /// Download project files specified in the manifest
    Download(DownloadArgs),

    /// Export the list of projects to a simple JSON file
    ExportJson(ExportJsonArgs),

    /// Display the current list of projects
    List(ListArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Target Minecraft version
    pub mc: String,

    /// Target platform (e.g., fabric, neoforge, paper)
    pub loader: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// ID of the project to remove
    pub id: String,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// List of project IDs or slugs to add
    pub ids: Vec<String>,

    /// Pin a specific version of the mod (default: latest)
    #[arg(short, long)]
    pub version: Option<String>,

    /// Restrict the mod to a specific side
    #[arg(short, long)]
    pub env: Option<EnvType>,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Directory where downloaded files will be saved
    #[arg(short, long, default_value = "mods")]
    pub output: PathBuf,

    /// Filter by environment (download only client, only server, or all)
    #[arg(short, long, value_enum, default_value_t = EnvType::Both)]
    pub env: EnvType,
}

#[derive(Args, Debug)]
pub struct ExportJsonArgs {
    /// Filter by environment
    #[arg(short, long, value_enum, default_value_t = EnvType::Both)]
    pub env: EnvType,

    /// Path to save the generated JSON export
    #[arg(short, long, default_value = "manifest.json")]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by environment
    #[arg(short, long, value_enum, default_value_t = EnvType::Both)]
    pub env: EnvType,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvType {
    Server,
    Client,
    Both,
}
