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
    pub value: u32,
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
    if n <= 50_000 {
        return Scalar {
            value: n as u32,
            unit: Unit::B,
        };
    } else if n > 50_000 && n <= 1_000_000 {
        return Scalar {
            value: (n / 1000) as u32,
            unit: Unit::KB,
        };
    } else if n > 1_000_000 && n <= 1_000_000_000 {
        return Scalar {
            value: (n / 1_000_000) as u32,
            unit: Unit::MB,
        };
    } else if n > 1_000_000_000 {
        return Scalar {
            value: (n / 1_000_000_000) as u32,
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
    pub size: Option<Scalar>,
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
}

const PARALLEL_THRESHOLD: usize = 64;
const MAX_RESULTS: usize = 100;

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
    let name = entry.file_name().to_string_lossy().into_owned();
    let modified: DateTime<Local> = entry.metadata().ok()?.modified().ok()?.into();
    let path = entry.path();
    let is_dir = path.is_dir();
    let size = if is_dir {
        None
    } else {
        Some(to_appropriate_unit(path.metadata().ok()?.len()))
    };
    Some(DirInfo {
        name: match method {
            Method::GoTo => name,
            Method::Search => path.to_string_lossy().into_owned(),
        },
        modified: match method {
            Method::GoTo => modified.format("%Y-%m-%d %H:%M").to_string(),
            Method::Search => String::new(),
        },
        path,
        is_dir,
        size,
    })
}

pub fn sync_get_dir(dir: &PathBuf, now: Option<Instant>) -> Result<Vec<DirInfo>, CError> {
    let entries: Vec<_> = read_dir(dir)
        .map_err(|_| CError::ReadDir)?
        .filter_map(|entry| entry.ok())
        .collect();

    let mut dirinfo = vec![DirInfo {
        name: "..".to_string(),
        modified: String::new(),
        path: dir.clone(),
        is_dir: true,
        size: None,
    }];
    dirinfo.extend(process_entries(&entries, &Method::GoTo));

    if let Some(now) = now {
        println!(
            "non streaming done in: {}μs; number of entries: {}; at directory: {}",
            now.elapsed().as_micros(),
            dirinfo.len(),
            dir.to_string_lossy()
        );
    }
    Ok(dirinfo)
}

pub async fn get_dir(dir: PathBuf, now: Option<Instant>) -> Result<Vec<DirInfo>, CError> {
    spawn_blocking(move || sync_get_dir(&dir, now))
        .await
        .map_err(|_| CError::Join)?
}

pub async fn delete_dir(dir: PathBuf) -> Result<(), CError> {
    spawn_blocking(move || sync_delete_dir(&dir))
        .await
        .map_err(|_| CError::Join)?
}

fn sync_delete_dir(dir: &PathBuf) -> Result<(), CError> {
    trash::delete(dir).map_err(|_| CError::Trash)
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
            let matches = path.to_string_lossy().to_ascii_lowercase().contains(query);

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
) -> Vec<DirInfo> {
    spawn_blocking(move || {
        if query.trim().is_empty() {
            return vec![previous_dir];
        }
        let mut filler_entry = vec![previous_dir];
        let entries = find_parallel(
            root,
            &query.to_ascii_lowercase().replace("/", "\\"),
            &AtomicUsize::new(0),
            max_results,
        );
        filler_entry.extend(process_entries(&entries, &Method::Search));
        filler_entry
    })
    .await
    .unwrap_or_default()
}
