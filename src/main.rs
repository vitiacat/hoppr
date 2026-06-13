use self::csv::{read_csv, write_csv};
use crate::cli::{AddArgs, EnvType, InitArgs, ListArgs, RemoveArgs};
use crate::csv::CsvMeta;
use crate::utils::{LoadProjectResult, find_version};
use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands, DownloadArgs, ExportJsonArgs};
use csv::CsvEntry;
use itertools::Itertools;
use modrinth::{fetch_projects, fetch_versions_for_project};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};
use std::fs::{self};

mod cli;
mod csv;
mod modrinth;
mod utils;

fn main() {
    let cli = Cli::parse();

    let error = match &cli.command {
        Commands::Init(args) => init(args, &cli),
        Commands::Add(args) => add(args, &cli),
        Commands::Remove(args) => remove(args, &cli),
        Commands::Download(args) => download(args, &cli),
        Commands::ExportJson(args) => export(args, &cli),
        Commands::List(args) => list(args, &cli),
    };

    if let Err(err) = error {
        eprintln!("❌ Error: {}", err);

        for cause in err.chain().skip(1) {
            eprintln!("  {} {}", "↳", cause);
        }

        std::process::exit(1);
    }
}

fn init(args: &InitArgs, cli: &Cli) -> anyhow::Result<()> {
    if cli.file.exists() {
        println!("❌ Manifest already exists at: {:?}", cli.file);
        return Ok(());
    }

    write_csv(
        &cli.file,
        &vec![],
        &CsvMeta {
            version: csv::CSV_VERSION,
            loader: args.loader.clone(),
            minecraft: args.mc.clone(),
            comments: vec![],
        },
    )
    .context("Failed to write csv")?;

    println!("✅ Created {:?} successfully", cli.file);
    Ok(())
}

fn add(args: &AddArgs, cli: &Cli) -> anyhow::Result<()> {
    let (mut entries, meta) = read_csv(&cli.file)?;

    let mut queue: VecDeque<(String, bool)> = args.ids.iter().map(|x| (x.clone(), true)).collect();
    let mut seen: HashSet<String> = entries.iter().map(|x| x.id.clone()).collect();
    let mut result: Vec<CsvEntry> = vec![];

    while let Some(x) = queue.pop_front() {
        let projects = fetch_projects(&[&x.0])?;
        if projects.is_empty() {
            println!("⚠️  Project {} not found on Modrinth", x.0);
            continue;
        }
        let project = &projects[0];
        println!("🔎 Searching for {}", project.title);

        if seen.contains(&project.id) || seen.contains(&project.slug) {
            continue;
        }

        let versions = fetch_versions_for_project(&project.id, &meta);
        if versions.is_empty() {
            println!("⚠️  Version for project {} not found", project.title);
            continue;
        }
        let version = &versions[0];
        seen.insert(x.0);

        version
            .dependencies
            .iter()
            .filter(|x| x.dependency_type == "required")
            .filter_map(|x| x.project_id.as_ref())
            .for_each(|x| {
                if seen.contains(x) {
                    return;
                }
                queue.push_back((x.clone(), false))
            });

        let env = match args.env {
            None => None,
            Some(EnvType::Client) => Some("c".to_string()),
            Some(EnvType::Server) => Some("s".to_string()),
            Some(EnvType::Both) => Some("cs".to_string()),
        };

        result.push(CsvEntry {
            id: project.slug.clone(),
            version: Some(version.version_number.clone()),
            environment: env,
        })
    }

    if result.is_empty() {
        println!("Nothing changed");
        return Ok(());
    }

    println!(
        "✅ Added {} projects: {}",
        result.len(),
        result.iter().map(|x| &x.id).join(", ")
    );

    entries.extend(result);

    write_csv(&cli.file, &entries, &meta)?;

    Ok(())
}

fn remove(args: &RemoveArgs, cli: &Cli) -> anyhow::Result<()> {
    let (mut entries, meta) = read_csv(&cli.file)?;
    let entry = entries.iter().find_position(|x| x.id == args.id);

    if let Some((index, entry)) = entry {
        println!(
            "✅ Removed project {} with version {:?}",
            entry.id, entry.version
        );
        entries.remove(index);
        write_csv(&cli.file, &entries, &meta)?;
    } else {
        println!("❌ Project {} not found", args.id);
    }

    Ok(())
}

fn download(args: &DownloadArgs, cli: &Cli) -> anyhow::Result<()> {
    let Some(LoadProjectResult { projects, meta, .. }) = utils::load_projects(&cli.file, args.env)
    else {
        return Ok(());
    };

    for (index, project) in projects.iter().enumerate() {
        println!(
            "🔎  {} ({}/{})",
            project.project.title,
            index + 1,
            projects.len()
        );

        let versions = fetch_versions_for_project(&project.project.id, &meta);
        let version = find_version(&project, &versions);

        if let Some(version) = version {
            if let Some(file) = version.files.first() {
                println!(
                    "📥 Downloading {} <{}, {}>",
                    project.project.title, version.version_number, version.version_type
                );
                utils::download_file(file, &args.output).context(format!(
                    "Failed to download project {}",
                    project.project.title
                ))?;
            }
        } else {
            println!("❌ Version not found: {:?}", project.csv_entry.version);
        }
    }

    println!(
        "✅ Downloaded {} files to: {:?}",
        projects.len(),
        args.output
    );

    Ok(())
}

fn export(args: &ExportJsonArgs, cli: &Cli) -> anyhow::Result<()> {
    let Some(LoadProjectResult { projects, meta, .. }) = utils::load_projects(&cli.file, args.env)
    else {
        return Ok(());
    };

    #[derive(Serialize)]
    struct JsonEntry {
        name: String,
        size: i64,
        sha1: String,
        url: String,
    }

    let mut json_entries = Vec::new();
    for (index, x) in projects.iter().enumerate() {
        let project = &x.project;
        println!("🔎  {} ({}/{})", project.title, index + 1, projects.len());

        let versions = fetch_versions_for_project(&project.id, &meta);
        let version = find_version(&x, &versions);

        if let Some(version) = version {
            if let Some(file) = version.files.first() {
                json_entries.push(JsonEntry {
                    name: file.filename.clone(),
                    sha1: file.hashes.sha1.clone(),
                    size: file.size,
                    url: file.url.clone(),
                });
            }
        }
    }

    fs::write(&args.output, serde_json::to_string(&json_entries)?)?;
    println!("💾 Exported to {:?}", args.output);

    Ok(())
}

fn list(args: &ListArgs, cli: &Cli) -> anyhow::Result<()> {
    let Some(LoadProjectResult {
        projects, filtered, ..
    }) = utils::load_projects(&cli.file, args.env)
    else {
        return Ok(());
    };

    println!(
        "List of mods ({}, filtered {}):",
        projects.len(),
        filtered.len()
    );
    println!(
        "{}",
        projects
            .into_iter()
            .map(|x| format!("{} ({}, {})", x.project.title, x.project.id, x.project.slug))
            .join(", ")
    );
    if !filtered.is_empty() {
        println!("Filtered:");
        println!(
            "{}",
            filtered
                .into_iter()
                .map(|x| format!("{} ({}, {})", x.0.title, x.0.id, x.0.slug))
                .join(", ")
        );
    }

    Ok(())
}
