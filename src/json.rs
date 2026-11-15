use crate::disks::DiskInfo;
use crate::info::Info;
use crate::monitor::Snapshot;
use std::fmt::Write;

/// Escapa una cadena como literal JSON (con comillas).
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Genera el documento JSON. `disks` es `None` si el módulo está oculto o agotó el tiempo.
pub fn render(version: &str, info: &Info, disks: Option<&[DiskInfo]>, snap: &Snapshot) -> String {
    let mut o = String::new();
    o.push_str("{\n");
    let _ = writeln!(o, "  \"version\": {},", quote(version));
    let _ = writeln!(o, "  \"user\": {},", quote(&info.user));
    let _ = writeln!(o, "  \"host\": {},", quote(&info.host));
    let _ = writeln!(o, "  \"distro\": {},", quote(&info.distro_id));
    o.push_str("  \"modules\": {");
    let rows: Vec<String> = info
        .sections
        .iter()
        .flat_map(|s| s.rows.iter())
        .map(|(id, _, v)| format!("\n    {}: {}", quote(id), quote(v)))
        .collect();
    o.push_str(&rows.join(","));
    o.push_str(if rows.is_empty() { "},\n" } else { "\n  },\n" });
    let _ = writeln!(o, "  \"cpu_usage\": {:.1},", snap.cpu * 100.0);
    let _ = writeln!(
        o,
        "  \"memory\": {{\"used\": {}, \"total\": {}, \"swap_used\": {}, \"swap_total\": {}}},",
        snap.used, snap.total, snap.swap_used, snap.swap_total
    );
    o.push_str("  \"disks\": [");
    let list: Vec<String> = disks
        .unwrap_or_default()
        .iter()
        .map(|d| {
            format!(
                "\n    {{\"mount\": {}, \"fs\": {}, \"used\": {}, \"total\": {}}}",
                quote(&d.mount),
                quote(&d.fs),
                d.used,
                d.total
            )
        })
        .collect();
    o.push_str(&list.join(","));
    o.push_str(if list.is_empty() { "]\n" } else { "\n  ]\n" });
    o.push_str("}\n");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info::Section;

    fn sample() -> (Info, Snapshot) {
        let info = Info {
            user: "ana".into(),
            host: "pc".into(),
            distro_id: "arch".into(),
            id_like: None,
            sections: vec![Section {
                title: "Sistema",
                rows: vec![("os", "OS", "Arch \"Linux\"".into())],
            }],
        };
        let snap = Snapshot {
            cpu: 0.256,
            used: 1,
            total: 4,
            swap_used: 0,
            swap_total: 2,
        };
        (info, snap)
    }

    #[test]
    fn quote_escapes_special_characters() {
        assert_eq!(quote("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
        assert_eq!(quote("\u{1}"), "\"\\u0001\"");
        assert_eq!(quote("ñandú"), "\"ñandú\"");
    }

    #[test]
    fn render_includes_modules_memory_and_disks() {
        let (info, snap) = sample();
        let disks = [DiskInfo {
            mount: "/".into(),
            fs: "ext4".into(),
            used: 5,
            total: 10,
        }];
        let out = render("0.4.0", &info, Some(&disks), &snap);
        assert!(out.contains("\"version\": \"0.4.0\""));
        assert!(out.contains("\"os\": \"Arch \\\"Linux\\\"\""));
        assert!(out.contains("\"cpu_usage\": 25.6"));
        assert!(out.contains("\"swap_total\": 2"));
        assert!(out.contains("{\"mount\": \"/\", \"fs\": \"ext4\", \"used\": 5, \"total\": 10}"));
        assert!(out.starts_with("{\n") && out.ends_with("}\n"));
    }

    #[test]
    fn render_without_modules_or_disks_stays_well_formed() {
        let (mut info, snap) = sample();
        info.sections.clear();
        let out = render("0.4.0", &info, None, &snap);
        assert!(out.contains("\"modules\": {},"));
        assert!(out.contains("\"disks\": []"));
    }
}
