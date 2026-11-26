//! Data libraries: catalog of official sources, download, conversion and the
//! local cache. Nothing here is redistributed with Guaraci: files are fetched
//! from the publisher on the user's machine.

use crate::decay::{read_decay, NuclideDecay};
use crate::elements::NuclideId;
use crate::photon::{read_photoatomic, ElementPhoton};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Version of the converted-data format. Bump when the stored structs change.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Photoatomic,
    Decay,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Photoatomic => "photoatomic",
            Kind::Decay => "decay",
        }
    }
}

/// One downloadable file of a library.
#[derive(Debug, Clone)]
pub struct Component {
    pub kind: Kind,
    /// Direct download URL, or `None` when the publisher blocks automatic downloads.
    pub url: Option<&'static str>,
    /// Page where a person can download the file by hand.
    pub page: &'static str,
    pub sha256: Option<&'static str>,
    /// Approximate size, bytes.
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub publisher: &'static str,
    pub components: &'static [Component],
}

pub static CATALOG: &[LibraryInfo] = &[
    LibraryInfo {
        id: "endfb8",
        title: "ENDF/B-VIII.0",
        publisher: "National Nuclear Data Center, Brookhaven National Laboratory",
        components: &[
            Component {
                kind: Kind::Photoatomic,
                url: Some("https://www.nndc.bnl.gov/endf-b8.0/zips/ENDF-B-VIII.0_photoat.zip"),
                page: "https://www.nndc.bnl.gov/endf-b8.0/download.html",
                sha256: Some("61aee2b8da51a79b3c0a68b46905443d0184f8750b5dc4c5644690b0248e835d"),
                size: 35_776_679,
            },
            Component {
                kind: Kind::Decay,
                url: Some("https://www.nndc.bnl.gov/endf-b8.0/zips/ENDF-B-VIII.0_decay.zip"),
                page: "https://www.nndc.bnl.gov/endf-b8.0/download.html",
                sha256: Some("9087395f6d74fe35b47bb4c8b37ef53b44e67569196c933f835a53cdf8d03d5e"),
                size: 14_173_710,
            },
        ],
    },
    LibraryInfo {
        id: "epics2017",
        title: "EPICS2017 (EPDL2017)",
        publisher: "IAEA Nuclear Data Section / LLNL",
        components: &[Component {
            kind: Kind::Photoatomic,
            url: None,
            page: "https://www-nds.iaea.org/epics/",
            sha256: None,
            size: 0,
        }],
    },
    LibraryInfo {
        id: "epdl97",
        title: "EPDL97",
        publisher: "IAEA Nuclear Data Section / LLNL",
        components: &[Component {
            kind: Kind::Photoatomic,
            url: None,
            page: "https://www-nds.iaea.org/epdl97/",
            sha256: None,
            size: 0,
        }],
    },
];

pub fn library_info(id: &str) -> Option<&'static LibraryInfo> {
    CATALOG.iter().find(|l| l.id.eq_ignore_ascii_case(id))
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("unknown data library `{0}`")]
    UnknownLibrary(String),
    #[error("{library} has no {kind} data")]
    NoComponent { library: String, kind: &'static str },
    #[error("{title} must be downloaded by hand from {page} (the site does not allow automatic downloads); then run `guaraci data install {id} --from <file>`")]
    ManualDownload {
        title: &'static str,
        page: &'static str,
        id: &'static str,
    },
    #[error("{title} ({kind}) is not installed; run `guaraci data install {id}`")]
    NotInstalled {
        title: &'static str,
        kind: &'static str,
        id: &'static str,
    },
    #[error("download of {url} failed: {msg}")]
    Download { url: String, msg: String },
    #[error("checksum mismatch for {what}: expected {expected}, got {got}")]
    Checksum {
        what: String,
        expected: String,
        got: String,
    },
    #[error("{0}")]
    Endf(#[from] crate::endf::EndfError),
    #[error("{path}: {err}")]
    Io { path: PathBuf, err: std::io::Error },
    #[error("cannot read archive {path}: {msg}")]
    Archive { path: PathBuf, msg: String },
    #[error("cached data in {path} is unreadable ({msg}); run `guaraci data install {id} --force`")]
    Cache { path: PathBuf, msg: String, id: String },
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> DataError + '_ {
    move |err| DataError::Io {
        path: path.to_path_buf(),
        err,
    }
}

/// Where converted data lives: `$GUARACI_DATA`, else the platform data dir
/// (`~/.local/share/guaraci` on Linux).
pub fn data_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("GUARACI_DATA") {
        return PathBuf::from(p);
    }
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("guaraci")
}

/// Record of where installed data came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub library: String,
    pub title: String,
    pub kind: Kind,
    pub source: String,
    pub sha256: String,
    pub installed: String,
    pub format_version: u32,
    pub entries: usize,
}

#[derive(Serialize, Deserialize)]
struct Stored<T> {
    provenance: Provenance,
    data: T,
}

pub type PhotonData = BTreeMap<u32, ElementPhoton>;
pub type DecayData = BTreeMap<NuclideId, NuclideDecay>;

fn cache_file(dir: &Path, lib: &str, kind: Kind) -> PathBuf {
    dir.join(lib).join(format!("{}.bin", kind.name()))
}

/// Is a component installed (and in the current format)?
pub fn installed(dir: &Path, lib: &str, kind: Kind) -> Option<Provenance> {
    let p = cache_file(dir, lib, kind);
    let f = std::fs::File::open(p).ok()?;
    // provenance is serialised first, so this reads only the start of the file
    let prov: Provenance = bincode::deserialize_from(std::io::BufReader::new(f)).ok()?;
    (prov.format_version == FORMAT_VERSION).then_some(prov)
}

/// Progress reporting for long operations.
pub trait Progress {
    fn message(&mut self, msg: &str);
    /// Bytes downloaded so far and total (0 if unknown).
    fn bytes(&mut self, done: u64, total: u64);
}

pub struct Silent;
impl Progress for Silent {
    fn message(&mut self, _: &str) {}
    fn bytes(&mut self, _: u64, _: u64) {}
}

/// Download (or take from `from`) one component, verify, convert and store it.
pub fn install(
    dir: &Path,
    lib: &str,
    kind: Kind,
    from: Option<&Path>,
    progress: &mut dyn Progress,
) -> Result<Provenance, DataError> {
    let info = library_info(lib).ok_or_else(|| DataError::UnknownLibrary(lib.to_string()))?;
    let comp = info
        .components
        .iter()
        .find(|c| c.kind == kind)
        .ok_or(DataError::NoComponent {
            library: info.title.to_string(),
            kind: kind.name(),
        })?;
    let target_dir = dir.join(info.id);
    std::fs::create_dir_all(&target_dir).map_err(io(&target_dir))?;

    let (archive, source) = match from {
        Some(p) => (p.to_path_buf(), p.display().to_string()),
        None => {
            let url = comp.url.ok_or(DataError::ManualDownload {
                title: info.title,
                page: comp.page,
                id: info.id,
            })?;
            let tmp = target_dir.join(format!("{}.download", kind.name()));
            progress.message(&format!("downloading {} {} data from {url}", info.title, kind.name()));
            download(url, &tmp, comp.size, progress)?;
            (tmp, url.to_string())
        }
    };

    progress.message("verifying checksum");
    let sha = sha256_file(&archive)?;
    if let Some(expected) = comp.sha256 {
        if from.is_none() && sha != expected {
            let _ = std::fs::remove_file(&archive);
            return Err(DataError::Checksum {
                what: source,
                expected: expected.into(),
                got: sha,
            });
        }
        if from.is_some() && sha != expected {
            progress.message(&format!(
                "note: {} differs from the catalogued file (sha256 {sha}); using it as given",
                archive.display()
            ));
        }
    }

    progress.message("converting");
    let result = convert_and_store(dir, info, kind, &archive, &source, &sha);
    if from.is_none() {
        let _ = std::fs::remove_file(&archive);
    }
    let result = result?;
    progress.message(&format!(
        "installed {} {} data ({} entries)",
        info.title,
        kind.name(),
        result.entries
    ));
    Ok(result)
}

fn convert_and_store(
    dir: &Path,
    info: &'static LibraryInfo,
    kind: Kind,
    archive: &Path,
    source: &str,
    sha: &str,
) -> Result<Provenance, DataError> {
    let installed_at = now_utc();
    let (archive, source, sha) = (archive.to_path_buf(), source, sha);
    let result = match kind {
        Kind::Photoatomic => {
            let data = convert_archive(&archive, |name, text| {
                if !name.ends_with(".endf") || name.contains("CHANGELOG") {
                    return Ok(None);
                }
                read_photoatomic(name, text).map(|e| Some((e.z, e)))
            })?;
            let prov = provenance(info, kind, source, sha, &installed_at, data.len());
            store(
                &cache_file(dir, info.id, kind),
                &Stored {
                    provenance: prov.clone(),
                    data,
                },
            )?;
            prov
        }
        Kind::Decay => {
            let data: DecayData = convert_archive(&archive, |name, text| {
                if !name.ends_with(".endf") {
                    return Ok(None);
                }
                Ok(read_decay(name, text)?.map(|d| (d.id, d)))
            })?;
            let prov = provenance(info, kind, source, sha, &installed_at, data.len());
            store(
                &cache_file(dir, info.id, kind),
                &Stored {
                    provenance: prov.clone(),
                    data,
                },
            )?;
            prov
        }
    };
    Ok(result)
}

fn provenance(info: &LibraryInfo, kind: Kind, source: &str, sha: &str, when: &str, n: usize) -> Provenance {
    Provenance {
        library: info.id.into(),
        title: info.title.into(),
        kind,
        source: source.into(),
        sha256: sha.into(),
        installed: when.into(),
        format_version: FORMAT_VERSION,
        entries: n,
    }
}

fn store<T: Serialize>(path: &Path, value: &T) -> Result<(), DataError> {
    let tmp = path.with_extension("tmp");
    let f = std::fs::File::create(&tmp).map_err(io(&tmp))?;
    bincode::serialize_into(std::io::BufWriter::new(f), value).map_err(|e| DataError::Io {
        path: tmp.clone(),
        err: std::io::Error::other(e),
    })?;
    std::fs::rename(&tmp, path).map_err(io(path))
}

fn load<T: for<'de> Deserialize<'de>>(
    dir: &Path,
    info: &'static LibraryInfo,
    kind: Kind,
) -> Result<(Provenance, T), DataError> {
    let path = cache_file(dir, info.id, kind);
    if installed(dir, info.id, kind).is_none() {
        return Err(DataError::NotInstalled {
            title: info.title,
            kind: kind.name(),
            id: info.id,
        });
    }
    let f = std::fs::File::open(&path).map_err(io(&path))?;
    let s: Stored<T> = bincode::deserialize_from(std::io::BufReader::new(f)).map_err(|e| DataError::Cache {
        path: path.clone(),
        msg: e.to_string(),
        id: info.id.into(),
    })?;
    Ok((s.provenance, s.data))
}

pub fn load_photon(dir: &Path, lib: &str) -> Result<(Provenance, PhotonData), DataError> {
    load(
        dir,
        library_info(lib).ok_or_else(|| DataError::UnknownLibrary(lib.into()))?,
        Kind::Photoatomic,
    )
}

pub fn load_decay(dir: &Path, lib: &str) -> Result<(Provenance, DecayData), DataError> {
    load(
        dir,
        library_info(lib).ok_or_else(|| DataError::UnknownLibrary(lib.into()))?,
        Kind::Decay,
    )
}

/// Read every file of a zip archive (or every file of a directory) through `f`.
fn convert_archive<K: Ord, V>(
    path: &Path,
    f: impl Fn(&str, &str) -> Result<Option<(K, V)>, crate::endf::EndfError>,
) -> Result<BTreeMap<K, V>, DataError> {
    let mut out = BTreeMap::new();
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .map_err(io(path))?
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_file() {
                let bytes = std::fs::read(&p).map_err(io(&p))?;
                let text = String::from_utf8_lossy(&bytes);
                if let Some((k, v)) = f(&p.display().to_string(), &text)? {
                    out.insert(k, v);
                }
            }
        }
        return Ok(out);
    }
    let file = std::fs::File::open(path).map_err(io(path))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| DataError::Archive {
        path: path.into(),
        msg: e.to_string(),
    })?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| DataError::Archive {
            path: path.into(),
            msg: e.to_string(),
        })?;
        if !entry.is_file() {
            continue;
        }
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(io(path))?;
        // ENDF is ASCII; a few files carry Latin-1 characters in comments
        let text = String::from_utf8_lossy(&bytes);
        if let Some((k, v)) = f(&name, &text)? {
            out.insert(k, v);
        }
    }
    Ok(out)
}

fn download(url: &str, dest: &Path, expected_size: u64, progress: &mut dyn Progress) -> Result<(), DataError> {
    let agent = ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .user_agent(concat!("guaraci/", env!("CARGO_PKG_VERSION")))
        .build();
    let resp = agent.get(url).call().map_err(|e| DataError::Download {
        url: url.into(),
        msg: e.to_string(),
    })?;
    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(expected_size);
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest).map_err(io(dest))?;
    let mut buf = vec![0u8; 1 << 16];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| DataError::Download {
            url: url.into(),
            msg: e.to_string(),
        })?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n]).map_err(io(dest))?;
        done += n as u64;
        progress.bytes(done, total);
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, DataError> {
    if path.is_dir() {
        return Ok("directory".into());
    }
    let mut f = std::fs::File::open(path).map_err(io(path))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(io(path))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Current UTC time as `YYYY-MM-DD hh:mm:ss UTC` (no extra dependency).
fn now_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // civil-from-days (H. Hinnant)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}
