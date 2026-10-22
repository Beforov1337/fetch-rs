use std::borrow::Cow;
use std::process::Command;

pub struct Logo {
    pub name: String,
    pub lines: Vec<String>,
    pub color_inner: String,
    pub color_outer: String,
}

pub const DEFAULT_GENTOO_LOGO: &[&str] = &[
    "         -/oyddmdhs+:.            ",
    "     -odNMMMMMMMMNNmhy+-`         ",
    "   -yNMMMMMMMMMMMNNNmmdhy+-       ",
    " `omMMMMMMMMMMMMNmdmmmmddhhy/`    ",
    " omMMMMMMMMMMMNhhyyyohmdddhhhdo`  ",
    ".ydMMMMMMMMMMdhs++so/smdddhhhhdm+`",
    " oyhdmNMMMMMMMNdyooydMddddhhhhyhNd.",
    "  :oyhhdNNMMMMMMMNNMMMdddhhhhhyymMh",
    "    .:+sydNMMMMMNNMMMMdddhhhhhhmMmy",
    "       /mMMMMMMNNNMMMdddhhhhhmMNhs:",
    "    `oNMMMMMMMNNNMMMddddhhdmMNhs+` ",
    "  `sNMMMMMMMMNNNMMMdddddmNMmhs/.   ",
    " /NMMMMMMMMNNNNMMMdddmNMNdso:`     ",
    "+MMMMMMMNNNNNMMMMdMNMNdso/-        ",
    "yMMNNNNNNNMMMMMNNMmhs+/-`          ",
    "/hMMNNNNNNNNMNdhs++/-`             ",
    "`/ohdmmddhys+++/:.`                ",
    "  `-//////:--.                     ",
];

impl Logo {
    pub fn default_gentoo() -> Self {
        Self {
            name: "gentoo".to_string(),
            lines: DEFAULT_GENTOO_LOGO.iter().map(|s| s.to_string()).collect(),
            color_inner: "\x1b[1;37m".to_string(),
            color_outer: "\x1b[1;35m".to_string(),
        }
    }

    pub fn load(name_opt: Option<&str>) -> Self {
        if let Some(name) = name_opt {
            if let Some(logo) = Self::try_load_fastfetch(name) {
                return logo;
            }
        } else {
            // Try detecting distro via /etc/os-release or fastfetch
            if let Some(distro) = detect_distro()
                && let Some(logo) = Self::try_load_fastfetch(&distro)
            {
                return logo;
            }
        }

        Self::default_gentoo()
    }

    fn try_load_fastfetch(distro: &str) -> Option<Self> {
        let output = Command::new("fastfetch")
            .args(["-c", "none", "-l", distro, "-s", "break", "--pipe", "false"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let mut lines = Vec::new();

        for line in raw.lines() {
            let trimmed = line.trim_end();
            // Truncate at cursor movements
            let mut cut = trimmed.len();
            let bytes = trimmed.as_bytes();
            let mut i = 0;
            while i + 2 < bytes.len() {
                if bytes[i] == 0x1b && bytes[i + 1] == b'[' {
                    let mut j = i + 2;
                    while j < bytes.len()
                        && ((bytes[j] >= b'0' && bytes[j] <= b'9') || bytes[j] == b';')
                    {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j] != b'm' {
                        cut = i;
                        break;
                    }
                    i = j;
                }
                i += 1;
            }

            let logo_line = &trimmed[..cut];
            if logo_line.is_empty() && lines.is_empty() {
                continue;
            }
            if cut < trimmed.len() && logo_line.is_empty() {
                break;
            }
            lines.push(logo_line.to_string());
        }

        while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
            lines.pop();
        }

        if lines.is_empty() {
            return None;
        }

        let (inner, outer) = get_distro_colors(distro);

        Some(Self {
            name: distro.to_string(),
            lines,
            color_inner: inner.to_string(),
            color_outer: outer.to_string(),
        })
    }
}

pub fn detect_distro() -> Option<String> {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                return Some(id.trim_matches('"').to_lowercase());
            }
        }
    }
    None
}

pub fn get_distro_colors(distro: &str) -> (&'static str, &'static str) {
    match distro.to_lowercase().as_str() {
        "arch" => ("\x1b[1;36m", "\x1b[1;34m"),
        "debian" => ("\x1b[1;31m", "\x1b[1;37m"),
        "ubuntu" => ("\x1b[1;33m", "\x1b[1;31m"),
        "fedora" => ("\x1b[1;34m", "\x1b[1;37m"),
        "nixos" => ("\x1b[1;36m", "\x1b[1;34m"),
        "void" => ("\x1b[1;32m", "\x1b[1;37m"),
        "alpine" => ("\x1b[1;34m", "\x1b[1;37m"),
        "gentoo" => ("\x1b[1;37m", "\x1b[1;35m"),
        _ => ("\x1b[1;37m", "\x1b[1;35m"),
    }
}

pub fn char_weight_utf8(ch: char) -> f32 {
    match ch {
        ' ' => 0.0,
        'M' => 1.00,
        'N' => 0.88,
        'm' => 0.76,
        'd' => 0.66,
        'h' | 'b' => 0.56,
        'y' => 0.46,
        'o' | 'n' => 0.38,
        's' => 0.30,
        '+' | '=' => 0.22,
        ':' => 0.18,
        '-' => 0.14,
        '/' => 0.12,
        '.' => 0.10,
        '`' => 0.08,
        '\'' => 0.06,
        '█' => 1.00,
        '▓' => 0.75,
        '▒' => 0.50,
        '░' => 0.25,
        c if c.is_ascii_uppercase() => 0.80,
        c if c.is_ascii_lowercase() => 0.50,
        c if c.is_ascii_digit() => 0.40,
        _ => 0.15,
    }
}

pub fn strip_ansi(s: &str) -> Cow<'_, str> {
    if !s.contains('\x1b') {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}
