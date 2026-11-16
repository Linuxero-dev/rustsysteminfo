use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 64 * 1024;
const MAX_VALUE: usize = 4096;
const NAME: &str = "cache.tsv";
const TTL: u64 = 86_400;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn stamp(m: &fs::Metadata) -> u64 {
    m.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn fingerprint(paths: &[&str]) -> String {
    let mut s = String::new();
    for p in paths {
        match fs::metadata(p) {
            Ok(m) => s.push_str(&format!("{p}:{}:{};", stamp(&m), m.len())),
            Err(_) => s.push_str(&format!("{p}:ausente;")),
        }
    }
    s
}

fn dir() -> Option<PathBuf> {
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|p| Path::new(p).is_absolute())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|p| Path::new(p).is_absolute())
                .map(|h| format!("{h}/.cache"))
        })?;
    Some(Path::new(&base).join("rsi"))
}

fn boot_id() -> String {
    fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read(path: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Ok(meta) = fs::metadata(path) else {
        return map;
    };
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return map;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return map;
    };
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('\t') {
            if !k.is_empty() && v.len() <= MAX_VALUE {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }
    map
}

pub struct Cache {
    path: Option<PathBuf>,
    boot: String,
    map: BTreeMap<String, String>,
}

impl Cache {
    pub fn load(enabled: bool) -> Self {
        let path = if enabled {
            dir().map(|d| d.join(NAME))
        } else {
            None
        };
        Self::open(path, boot_id())
    }

    fn open(path: Option<PathBuf>, boot: String) -> Self {
        let mut map = path.as_deref().map(read).unwrap_or_default();
        if map.get("boot").map(String::as_str) != Some(boot.as_str()) {
            map.clear();
        }
        Self { path, boot, map }
    }

    pub fn hit(&self, key: &str, fp: &str) -> Option<String> {
        if self.path.is_none() || self.age(key)? >= TTL {
            return None;
        }
        if self.map.get(&format!("{key}.fp")).map(String::as_str) != Some(fp) {
            return None;
        }
        self.map.get(key).cloned()
    }

    pub fn store(&mut self, key: &str, fp: &str, value: &str) {
        if self.path.is_none() || value.len() > MAX_VALUE {
            return;
        }
        self.map.insert(key.to_string(), value.to_string());
        self.map.insert(format!("{key}.fp"), fp.to_string());
        self.map.insert(format!("{key}.ts"), now().to_string());
    }

    pub fn age(&self, key: &str) -> Option<u64> {
        let ts: u64 = self.map.get(&format!("{key}.ts"))?.parse().ok()?;
        Some(now().saturating_sub(ts))
    }

    pub fn save(&self) {
        let Some(path) = &self.path else { return };
        let Some(parent) = path.parent() else { return };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let mut text = format!("boot\t{}\n", self.boot);
        for (k, v) in &self.map {
            if k != "boot" && !k.contains(['\t', '\n']) && !v.contains(['\t', '\n']) {
                text.push_str(&format!("{k}\t{v}\n"));
            }
        }
        if text.len() as u64 > MAX_BYTES {
            return;
        }
        let tmp = path.with_extension("tmp");
        if fs::write(&tmp, text).is_ok() {
            let _ = fs::rename(&tmp, path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rsi-cache-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn at(d: &Path) -> Cache {
        Cache::open(Some(d.join(NAME)), "b".into())
    }

    #[test]
    fn hit_requires_the_same_fingerprint() {
        let d = tmp("fp");
        let mut c = at(&d);
        c.store("k", "uno", "v");
        assert_eq!(c.hit("k", "uno").as_deref(), Some("v"));
        assert_eq!(c.hit("k", "dos"), None);
        assert_eq!(c.hit("otra", "uno"), None);
    }

    #[test]
    fn values_survive_a_reload() {
        let d = tmp("reload");
        let mut c = at(&d);
        c.store("k", "fp", "2500 (rpm)");
        c.save();
        assert_eq!(at(&d).hit("k", "fp").as_deref(), Some("2500 (rpm)"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_different_boot_discards_everything() {
        let d = tmp("boot");
        let mut c = at(&d);
        c.store("k", "fp", "v");
        c.save();
        let other = Cache::open(Some(d.join(NAME)), "otro".into());
        assert_eq!(other.hit("k", "fp"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn corrupt_or_oversized_files_are_ignored() {
        let d = tmp("corrupt");
        fs::create_dir_all(&d).unwrap();
        let f = d.join(NAME);
        fs::write(&f, "basura sin tabuladores\n").unwrap();
        assert_eq!(at(&d).hit("k", "fp"), None);
        fs::write(&f, vec![b'x'; 70 * 1024]).unwrap();
        assert_eq!(at(&d).hit("k", "fp"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_disabled_cache_never_hits_or_writes() {
        let d = tmp("off");
        let mut c = Cache::open(None, "b".into());
        c.store("k", "fp", "v");
        c.save();
        assert_eq!(c.hit("k", "fp"), None);
        assert!(!d.exists());
    }

    #[test]
    fn values_with_tabs_are_not_written() {
        let d = tmp("tabs");
        let mut c = at(&d);
        c.store("k", "fp", "a\tb");
        c.save();
        assert_eq!(at(&d).hit("k", "fp"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn fingerprint_tracks_size_and_existence() {
        let d = tmp("finger");
        fs::create_dir_all(&d).unwrap();
        let f = d.join("db");
        let p = f.to_string_lossy().into_owned();
        let ausente = fingerprint(&[&p]);
        fs::write(&f, "a").unwrap();
        let uno = fingerprint(&[&p]);
        fs::write(&f, "ab").unwrap();
        assert_ne!(ausente, uno);
        assert_ne!(uno, fingerprint(&[&p]));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn age_is_zero_just_after_storing() {
        let d = tmp("age");
        let mut c = at(&d);
        assert_eq!(c.age("k"), None);
        c.store("k", "fp", "v");
        assert_eq!(c.age("k"), Some(0));
    }
}
