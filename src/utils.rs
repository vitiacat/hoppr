use crate::cli::EnvType;
use crate::csv::{CsvEntry, CsvMeta};
use crate::modrinth;
use crate::modrinth::{ModVersion, Project, VersionFile};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use itertools::Itertools;

pub struct CsvProjectEntry {
    pub project: Project,
    pub csv_entry: CsvEntry,
}

pub struct LoadProjectResult {
    pub projects: Vec<CsvProjectEntry>,
    pub filtered: Vec<(Project, CsvEntry)>,
    pub meta: CsvMeta,
}

pub fn load_projects(file_path: &PathBuf, side: EnvType) -> Option<LoadProjectResult> {
    let (slugs, meta) = crate::csv::read_csv(file_path).expect("Extract failed");
    if slugs.is_empty() {
        println!("No valid Modrinth URLs found in the file.");
        return None;
    }

    let ids = slugs.iter().map(|x| &x.id).collect_vec();
    let projects = match modrinth::fetch_projects(ids.as_slice()) {
        Ok(val) => val,
        Err(e) => {
            println!("Failed to fetch projects: {}", e);
            return None;
        }
    };

    let mut projects_map= HashMap::new();
    for project in &projects {
        projects_map.insert(project.id.as_str(), project);
        projects_map.insert(project.slug.as_str(), project);
    }

    let paired_projects = slugs
        .into_iter()
        .filter_map(|entry| {
            let project = match projects_map.get(entry.id.as_str()) {
                Some(&project) => Some(project.clone()),
                None => {
                    eprintln!("⚠ Project with ID/slug '{}' not found in API response.", entry.id);
                    None
                }
            };

            project.map(|p| (p, entry))
        })
        .collect_vec();

    // filter by env
    let (filtered, excluded): (Vec<(Project, CsvEntry)>, Vec<(Project, CsvEntry)>) =
        paired_projects.into_iter().partition(|(project, entry)| {
            let env_override = entry
                .environment
                .as_deref()
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_default();

            if !env_override.is_empty() {
                match side {
                    EnvType::Client => env_override.contains('c'),
                    EnvType::Server => env_override.contains('s'),
                    EnvType::Both => true,
                }
            } else {
                let client_status = &project.client_side;
                let server_status = &project.server_side;

                if client_status == "unknown" && server_status == "unknown" {
                    println!("⚠ {} has no information about env", project.title);
                }

                match side {
                    EnvType::Client => client_status != "unsupported",
                    EnvType::Server => server_status != "unsupported",
                    EnvType::Both => true,
                }
            }
        });

    let selected_projects = filtered
        .into_iter()
        .map(|(project, csv_entry)| CsvProjectEntry { project, csv_entry })
        .collect_vec();

    Some(LoadProjectResult {
        projects: selected_projects,
        filtered: excluded,
        meta,
    })
}

pub fn download_file(file: &VersionFile, output_path: &PathBuf) -> Result<()> {
    let pb = ProgressBar::new(file.size as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} - {msg} ({eta})")?
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("#>-"),
    );

    pb.set_message(file.filename.clone());

    let path = output_path.join(&file.filename);
    if path.exists() {
        println!("Skipping {:?} (already exists)", path);
        return Ok(());
    }

    let response = ureq::get(&file.url).call()?;
    let mut reader = response.into_body().into_reader();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create destination directory")?;
    }

    let mut out_file = File::create(&path).context(format!("Failed to open path {:?}", path))?;

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        out_file.write_all(&buffer[..bytes_read])?;
        pb.inc(bytes_read as u64);
    }

    Ok(())
}

pub fn find_version<'a>(
    project: &CsvProjectEntry,
    versions: &'a [ModVersion],
) -> Option<&'a ModVersion> {
    if let Some(v) = &project.csv_entry.version {
        versions
            .iter()
            .find(|x| x.version_number == *v || x.id == *v)
    } else {
        versions.first() // or latest
    }
}
