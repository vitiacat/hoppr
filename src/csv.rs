use anyhow::Context;
use csv::WriterBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub const CSV_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvEntry {
    pub id: String,
    pub version: Option<String>,
    pub environment: Option<String>,

    #[serde(skip)]
    pub is_version_fixed: bool
}

#[derive(Debug)]
pub struct CsvMeta {
    pub minecraft: String,
    pub loader: String,
    pub version: u32,
    pub comments: Vec<String>,
}

pub fn read_csv(file_path: &PathBuf) -> anyhow::Result<(Vec<CsvEntry>, CsvMeta)> {
    let file = File::open(file_path).context("Failed to open manifest file")?;
    let reader = BufReader::new(file);

    let mut meta = CsvMeta::default();
    let mut csv_data = String::new();

    for line_result in reader.lines() {
        let line = line_result?;
        if line.starts_with('#') {
            let clean = line.trim_start_matches('#').trim();
            if let Some((key, val)) = clean.split_once(':') {
                match key.trim() {
                    "minecraft" => meta.minecraft = val.trim().to_string(),
                    "loader" => meta.loader = val.trim().to_string(),
                    "version" => meta.version = val.trim().parse().context("Failed to parse manifest version")?,
                    _ => meta.comments.push(line),
                }
            } else {
                meta.comments.push(line);
            }
        } else {
            csv_data.push_str(&line);
            csv_data.push('\n');
        }
    }

    let mut entries = vec![];
    let regex = Regex::new(r"modrinth\.com/mod/(.*)")?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    for result in csv_reader.deserialize() {
        let mut record: CsvEntry = result.context("Failed to deserialize manifest")?;
        if record.id.starts_with("http") {
            record.id = regex
                .captures(&record.id)
                .map(|captures| String::from(captures.get(1).unwrap().as_str()))
                .unwrap_or(record.id);
        }
        if let Some(version) = record.version.as_deref().and_then(|x| x.strip_prefix('=')) {
            record.is_version_fixed = true;
            record.version = Some(version.to_string());
        }

        entries.push(record)
    }

    Ok((entries, meta))
}

pub fn write_csv(
    file_path: &PathBuf,
    entries: Vec<CsvEntry>,
    meta: &CsvMeta,
) -> anyhow::Result<()> {
    let mut file =
        File::create(file_path).context("Failed to open manifest file in write-only mode")?;

    writeln!(file, "# version: {}", meta.version)?;
    writeln!(file, "# minecraft: {}", meta.minecraft)?;
    writeln!(file, "# loader: {}", meta.loader)?;

    {
        let mut wtr = WriterBuilder::new().from_writer(&file);

        for mut entry in entries {
            if entry.is_version_fixed {
                entry.version = entry.version.map(|x| format!("={x}"));
            }
            wtr.serialize(entry)?;
        }

        wtr.flush()?;
    }

    for comment in &meta.comments {
        writeln!(file, "{}", comment)?;
    }

    Ok(())
}

impl Default for CsvMeta {
    fn default() -> Self {
        Self {
            minecraft: "".to_string(),
            loader: "".to_string(),
            version: CSV_VERSION,
            comments: vec![],
        }
    }
}
