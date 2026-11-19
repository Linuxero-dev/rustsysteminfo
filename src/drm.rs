//! Modo de vídeo activo (resolución y frecuencia reales) preguntando al kernel
//! por DRM. El EDID solo dice qué prefiere el monitor, no qué se está usando.
//!
//! Solo se hacen consultas de lectura que no provocan sondeo del conector
//! (`GETCONNECTOR` con `count_modes = 1`, como `drmModeGetConnectorCurrent`).
//! Cualquier fallo (sin permisos, por SSH, kernel raro) devuelve `None` y el
//! llamador usa el EDID.

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

/// `drm_mode_modeinfo`: 68 bytes.
const MODEINFO: usize = 68;
/// `drm_mode_get_connector`: 80 bytes. `drm_mode_get_encoder`: 20. `drm_mode_crtc`: 104.
const CONNECTOR: usize = 80;
const ENCODER: usize = 20;
const CRTC: usize = 104;

const IOCTL_GETCRTC: u32 = 0xC068_64A1;
const IOCTL_GETENCODER: u32 = 0xC014_64A6;
const IOCTL_GETCONNECTOR: u32 = 0xC050_64A7;

const FLAG_INTERLACE: u32 = 1 << 4;
const FLAG_DBLSCAN: u32 = 1 << 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub hz: u32,
}

fn u16_at(b: &[u8], o: usize) -> u32 {
    u32::from(u16::from_le_bytes([b[o], b[o + 1]]))
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// Interpreta un `drm_mode_modeinfo` (68 bytes). La frecuencia se calcula con
/// el reloj de píxel y los totales, no con el campo `vrefresh` (entero y
/// redondeado hacia abajo en algunos controladores).
pub fn parse_modeinfo(b: &[u8]) -> Option<Mode> {
    if b.len() < MODEINFO {
        return None;
    }
    let clock_khz = u64::from(u32_at(b, 0));
    let (width, htotal) = (u16_at(b, 4), u16_at(b, 10));
    let (height, vtotal, vscan) = (u16_at(b, 14), u16_at(b, 20), u16_at(b, 22));
    let flags = u32_at(b, 28);
    let mut total = u64::from(htotal) * u64::from(vtotal);
    if vscan > 1 {
        total *= u64::from(vscan);
    }
    if clock_khz == 0 || total == 0 || width == 0 || height == 0 {
        return None;
    }
    // Como `drm_mode_vrefresh` del kernel, en miliHz: entrelazado x2, doble barrido /2.
    let mut mhz = clock_khz * 1_000_000 / total;
    if flags & FLAG_INTERLACE != 0 {
        mhz *= 2;
    }
    if flags & FLAG_DBLSCAN != 0 {
        mhz /= 2;
    }
    let hz = (mhz + 500) / 1000;
    (10..=1000).contains(&hz).then_some(Mode {
        width,
        height,
        hz: hz as u32,
    })
}

fn put32(buf: &mut [u8], o: usize, v: u32) {
    buf[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn ioctl(fd: i32, req: u32, buf: &mut [u8]) -> bool {
    // SAFETY: `buf` es un búfer vivo del tamaño exacto de la estructura del
    // ioctl; el kernel solo escribe dentro de ella (y del búfer de modos que
    // `connector_crtc` enlaza, de tamaño suficiente para `count_modes = 1`).
    unsafe { libc::ioctl(fd, req as _, buf.as_mut_ptr()) == 0 }
}

/// Modo activo del conector `connector_id` de `/dev/dri/<card>` (p. ej. `card0`).
pub fn active_mode(card: &str, connector_id: u32) -> Option<Mode> {
    if !card.starts_with("card") || !card[4..].bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let path = format!("/dev/dri/{card}");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .or_else(|_| OpenOptions::new().read(true).open(&path))
        .ok()?;
    let fd = file.as_raw_fd();

    // Conector -> encoder actual. `count_modes = 1` evita que el kernel sondee el conector.
    let mut modes = [0u8; MODEINFO];
    let mut con = [0u8; CONNECTOR];
    con[8..16].copy_from_slice(&(modes.as_mut_ptr() as u64).to_le_bytes()); // modes_ptr
    put32(&mut con, 32, 1); // count_modes
    put32(&mut con, 48, connector_id);
    if !ioctl(fd, IOCTL_GETCONNECTOR, &mut con) {
        return None;
    }
    let encoder_id = u32_at(&con, 44);
    if encoder_id == 0 {
        return None;
    }

    // Encoder -> CRTC.
    let mut enc = [0u8; ENCODER];
    put32(&mut enc, 0, encoder_id);
    if !ioctl(fd, IOCTL_GETENCODER, &mut enc) {
        return None;
    }
    let crtc_id = u32_at(&enc, 8);
    if crtc_id == 0 {
        return None;
    }

    // CRTC -> modo.
    let mut crtc = [0u8; CRTC];
    put32(&mut crtc, 12, crtc_id);
    if !ioctl(fd, IOCTL_GETCRTC, &mut crtc) || u32_at(&crtc, 32) == 0 {
        return None;
    }
    parse_modeinfo(&crtc[36..36 + MODEINFO])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(clock_khz: u32, h: (u16, u16), v: (u16, u16), vscan: u16, flags: u32) -> Vec<u8> {
        let mut b = vec![0u8; MODEINFO];
        b[0..4].copy_from_slice(&clock_khz.to_le_bytes());
        b[4..6].copy_from_slice(&h.0.to_le_bytes());
        b[10..12].copy_from_slice(&h.1.to_le_bytes());
        b[14..16].copy_from_slice(&v.0.to_le_bytes());
        b[20..22].copy_from_slice(&v.1.to_le_bytes());
        b[22..24].copy_from_slice(&vscan.to_le_bytes());
        b[28..32].copy_from_slice(&flags.to_le_bytes());
        b
    }

    #[test]
    fn computes_refresh_from_the_pixel_clock() {
        // 1920x1080@60 (CEA): 148.5 MHz, 2200x1125.
        let m = parse_modeinfo(&mode(148_500, (1920, 2200), (1080, 1125), 0, 0)).unwrap();
        assert_eq!((m.width, m.height, m.hz), (1920, 1080, 60));
        // 1600x900@75 (CVT-RB): ~ 119 MHz, 1760x926 -> ~73-75; 1760*926*75 = 122.2 MHz.
        let m = parse_modeinfo(&mode(122_220, (1600, 1760), (900, 926), 0, 0)).unwrap();
        assert_eq!(m.hz, 75);
        // 59.94 Hz se muestra como 60.
        let m = parse_modeinfo(&mode(148_352, (1920, 2200), (1080, 1125), 0, 0)).unwrap();
        assert_eq!(m.hz, 60);
    }

    #[test]
    fn interlaced_and_doublescan_modes_are_adjusted() {
        // 1080i60: 74.25 MHz, 2200x1125 por imagen completa de 2 campos -> 60 campos/s.
        let m =
            parse_modeinfo(&mode(74_250, (1920, 2200), (1080, 1125), 0, FLAG_INTERLACE)).unwrap();
        assert_eq!(m.hz, 60);
        // doble barrido: 640x400 a 70 Hz de barrido real = 35 Hz mostrados.
        let m = parse_modeinfo(&mode(25_175, (640, 800), (400, 449), 0, FLAG_DBLSCAN)).unwrap();
        assert_eq!(m.hz, 35);
    }

    #[test]
    fn rejects_empty_or_absurd_modes() {
        assert!(parse_modeinfo(&[0u8; MODEINFO]).is_none());
        assert!(parse_modeinfo(&[0u8; 10]).is_none());
        assert!(parse_modeinfo(&mode(1, (1920, 2200), (1080, 1125), 0, 0)).is_none());
    }

    #[test]
    fn invalid_card_names_are_never_opened() {
        assert!(active_mode("../etc/passwd", 1).is_none());
        assert!(active_mode("card", 1).is_none());
        assert!(active_mode("card0-DP-1", 1).is_none());
        assert!(active_mode("card99", 1).is_none());
    }
}
