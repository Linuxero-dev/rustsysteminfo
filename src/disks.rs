use crate::ui::{bar, clean, dim, human, pct_color, reset, trunc};
use std::collections::HashSet;
use sysinfo::Disks;

const SKIP_FS: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "devpts",
    "squashfs",
    "overlay",
    "efivarfs",
    "ramfs",
    "proc",
    "sysfs",
    "cgroup2",
    "autofs",
    "binfmt_misc",
    "debugfs",
    "tracefs",
    "securityfs",
    "nsfs",
    "rpc_pipefs",
    "fusectl",
    "configfs",
    "hugetlbfs",
    "mqueue",
    "pstore",
    "bpf",
    "fuse.portal",
    "fuse.gvfsd-fuse",
];
const SKIP_MOUNT: &[&str] = &[
    "/snap",
    "/var/lib/docker",
    "/var/lib/containers",
    "/run/docker",
];

pub struct DiskInfo {
    mount: String,
    fs: String,
    used: u64,
    total: u64,
}

pub fn collect() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut seen = HashSet::new();
    disks
        .list()
        .iter()
        .filter_map(|d| {
            let total = d.total_space();
            let fs = d.file_system().to_string_lossy().into_owned();
            let mount = d.mount_point().to_string_lossy().into_owned();
            let skip = total == 0
                || SKIP_FS.contains(&fs.as_str())
                || SKIP_MOUNT.iter().any(|p| mount.starts_with(p));
            (!skip && seen.insert((d.name().to_owned(), total))).then(|| DiskInfo {
                mount: clean(&mount),
                fs: clean(&fs),
                used: total.saturating_sub(d.available_space()),
                total,
            })
        })
        .collect()
}

fn label(mount: &str) -> String {
    if mount.chars().count() <= 11 {
        return mount.to_string();
    }
    trunc(
        &format!("…/{}", mount.rsplit('/').next().unwrap_or(mount)),
        11,
    )
}

pub fn lines(disks: &[DiskInfo]) -> Vec<String> {
    let (d, r) = (dim(), reset());
    if disks.is_empty() {
        return vec![format!("{d}Sin discos disponibles{r}")];
    }
    disks
        .iter()
        .map(|k| {
            let f = k.used as f64 / k.total as f64;
            format!(
                "{:<12}{d}{:<6}{r} {} {}{:>3.0}%{r} {} / {}",
                label(&k.mount),
                trunc(&k.fs, 5),
                bar(f, 14),
                pct_color(f * 100.0),
                f * 100.0,
                human(k.used, 1),
                human(k.total, 1),
            )
        })
        .collect()
}
