use crate::gui::LoadedEntries;
use chrono::{DateTime, Local};
use rayon::prelude::*;
use std::{
    fs::{DirEntry, read_dir},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use tokio::task::spawn_blocking;

#[derive(Default, Debug, Clone)]
pub enum Unit {
    #[default]
    B,
    KB,
    MB,
    GB,
}

#[derive(Default, Debug, Clone)]
pub struct Scalar {
    pub value: u64,
    pub unit: Unit,
}
impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Unit::B => "B",
            Unit::KB => "kB",
            Unit::MB => "MB",
            Unit::GB => "GB",
        };
        write!(f, "{}", s)
    }
}

fn to_appropriate_unit(size_in_bytes: u64) -> Scalar {
    let n = size_in_bytes;
    if n <= 25_000 {
        return Scalar {
            value: n,
            unit: Unit::B,
        };
    } else if n > 25_000 && n <= 1_000_000 {
        return Scalar {
            value: (n / 1000) as u64,
            unit: Unit::KB,
        };
    } else if n > 1_000_000 && n <= 1_000_000_000 {
        return Scalar {
            value: (n / 1_000_000) as u64,
            unit: Unit::MB,
        };
    } else if n > 1_000_000_000 {
        return Scalar {
            value: (n / 1_000_000_000) as u64,
            unit: Unit::GB,
        };
    } else {
        return Scalar::default();
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirInfo {
    pub name: String,
    pub modified: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: (String, u64),
}

impl DirInfo {
    pub fn new(
        name: String,
        modified: String,
        path: PathBuf,
        is_dir: bool,
        size: (String, u64),
    ) -> Self {
        Self {
            name,
            modified,
            path,
            is_dir,
            size,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Method {
    GoTo,
    Search,
}

#[derive(Debug, Clone)]
pub enum CError {
    ReadDir,
    Join,
    Trash,
    Move,
}

const PARALLEL_THRESHOLD: usize = 150;

fn process_entries(entries: &[DirEntry], method: &Method) -> Vec<DirInfo> {
    if entries.len() > PARALLEL_THRESHOLD {
        entries
            .par_iter()
            .filter_map(|entry| process_entry(entry, method))
            .collect()
    } else {
        entries
            .iter()
            .filter_map(|entry| process_entry(entry, method))
            .collect()
    }
}

fn process_entry(entry: &DirEntry, method: &Method) -> Option<DirInfo> {
    match method {
        Method::GoTo => {
            let metadata = entry.metadata().ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if is_hidden(&file_name, &metadata) {
                return None;
            }
            let is_dir = metadata.is_dir();
            let name = if is_dir {
                format!("📁 {}", file_name)
            } else {
                format!("{}", file_name)
            };
            let modified: DateTime<Local> = metadata.modified().ok()?.into();
            let (numerical_size, display_size) = if is_dir {
                (0, String::new())
            } else {
                let scalar = to_appropriate_unit(metadata.len());
                (metadata.len(), format!("{}{}", scalar.value, scalar.unit))
            };
            let path = entry.path();
            Some(DirInfo::new(
                name,
                modified.format("%Y-%m-%d %H-%M").to_string(),
                path,
                is_dir,
                (display_size, numerical_size),
            ))
        }
        Method::Search => {
            let file_path = entry.path();
            let file_path = file_path.to_string_lossy();
            let metadata = entry.metadata().ok()?;
            let is_dir = metadata.is_dir();
            let path = entry.path();
            let name = if is_dir {
                format!("📁 {}", file_path)
            } else {
                file_path.into_owned()
            };

            let (numerical_size, display_size) = if is_dir {
                (0, String::new())
            } else {
                let scalar = to_appropriate_unit(metadata.len());
                (metadata.len(), format!("{}{}", scalar.value, scalar.unit))
            };

            Some(DirInfo {
                name,
                is_dir,
                size: (display_size, numerical_size),
                path: path,
                ..Default::default()
            })
        }
    }
}

pub fn sync_get_dir(dir: &PathBuf, now: Option<Instant>) -> Result<LoadedEntries, CError> {
    let entries: Vec<_> = read_dir(&dir)
        .map_err(|_| CError::ReadDir)?
        .filter_map(|entry| entry.ok())
        .collect();

    let root_dir = DirInfo {
        name: "..".to_string(),
        modified: String::new(),
        path: dir.clone(),
        is_dir: true,
        ..Default::default()
    };
    let dirinfo: Vec<DirInfo> = process_entries(&entries, &Method::GoTo);

    if let Some(now) = now {
        println!(
            "proccesed entries in: {}μs; number of entries: {}; at directory: {}",
            now.elapsed().as_micros(),
            dirinfo.len(),
            dir.to_string_lossy()
        );
    }

    let loaded_entires = LoadedEntries::new(root_dir, dirinfo);

    Ok(loaded_entires)
}

pub async fn get_dir(dir: PathBuf, now: Option<Instant>) -> Result<LoadedEntries, CError> {
    spawn_blocking(move || sync_get_dir(&dir, now))
        .await
        .map_err(|_| CError::Join)?
}

pub async fn delete_dir(dir: PathBuf) -> Result<(), CError> {
    spawn_blocking(move || trash::delete(&dir).map_err(|_| CError::Trash))
        .await
        .map_err(|_| CError::Join)?
}

fn find_parallel(root: PathBuf, query: &str, found: &AtomicUsize, limit: usize) -> Vec<DirEntry> {
    if found.load(Ordering::Relaxed) >= limit {
        return vec![];
    }

    let Ok(entries) = read_dir(root) else {
        return vec![];
    };
    let entries: Vec<_> = entries.flatten().collect();

    entries
        .into_par_iter()
        .flat_map_iter(|entry| {
            let count = found.load(Ordering::Relaxed);
            if count >= limit {
                return vec![];
            }
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let matches = path
                .to_string_lossy()
                .as_bytes()
                .windows(query.len())
                .any(|w| w.eq_ignore_ascii_case(query.as_bytes()));

            let mut results = vec![];
            if matches {
                found.fetch_add(1, Ordering::Relaxed);
                results.push(entry);
            }

            if is_dir {
                results.extend(find_parallel(path, query, found, limit));
            }

            results
        })
        .collect()
}

pub async fn search_all(
    query: String,
    root: PathBuf,
    previous_dir: DirInfo,
    max_results: usize,
) -> LoadedEntries {
    spawn_blocking(move || {
        if query.trim().is_empty() {
            return LoadedEntries::new(DirInfo::default(), vec![previous_dir]);
        }

        let entries = find_parallel(
            root,
            &&query.to_ascii_lowercase().replace("/", "\\"),
            &AtomicUsize::new(0),
            max_results,
        );
        let dir_entries = process_entries(&entries, &Method::Search);

        LoadedEntries::new(previous_dir, dir_entries)
    })
    .await
    .unwrap_or_default()
}

fn is_hidden(name: &str, metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
            return true;
        }
    }
    name.starts_with('.')
}

pub async fn move_entry(source: PathBuf, dest_dir: PathBuf) -> Result<(), CError> {
    spawn_blocking(move || {
        let file_name = source.file_name().ok_or(CError::ReadDir)?;
        let dest = dest_dir.join(file_name);
        std::fs::rename(&source, &dest).map_err(|_| CError::Move)
    })
    .await
    .map_err(|_| CError::Join)?
}

pub async fn rename_entry(source: PathBuf, dest: PathBuf) -> Result<(), CError> {
    spawn_blocking(move || std::fs::rename(&source, &dest).map_err(|_| CError::Move))
        .await
        .map_err(|_| CError::Join)?
}
