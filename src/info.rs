use std::fs;

pub struct SysInfo {
    pub lines: Vec<(String, String)>,
    pub title: String,
    pub title_len: usize,
}

impl SysInfo {
    pub fn gather() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let hostname = fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "localhost".to_string());
        let title_len = username.len() + 1 + hostname.len();
        let title = format!("\x1b[1;32m{username}\x1b[0m@\x1b[1;32m{hostname}\x1b[0m");

        let mut lines = Vec::new();

        if let Some(os) = gather_os() {
            lines.push(("OS".to_string(), os));
        }
        if let Some(host) = gather_host() {
            lines.push(("Host".to_string(), host));
        }
        if let Some(kernel) = gather_kernel() {
            lines.push(("Kernel".to_string(), kernel));
        }
        if let Some(uptime) = gather_uptime() {
            lines.push(("Uptime".to_string(), uptime));
        }
        if let Some(pkgs) = gather_packages() {
            lines.push(("Packages".to_string(), pkgs));
        }
        if let Some(shell) = gather_shell() {
            lines.push(("Shell".to_string(), shell));
        }
        if let Some(wm) = gather_wm() {
            lines.push(("WM".to_string(), wm));
        }
        if let Some(terminal) = gather_terminal() {
            lines.push(("Terminal".to_string(), terminal));
        }
        if let Some(cpu) = gather_cpu() {
            lines.push(("CPU".to_string(), cpu));
        }
        if let Some(mem) = gather_memory() {
            lines.push(("Memory".to_string(), mem));
        }

        Self {
            lines,
            title,
            title_len,
        }
    }

    pub fn max_line_width(&self) -> usize {
        let mut max_w = self.title_len;
        for (label, val) in &self.lines {
            let w = label.len() + 2 + val.len();
            if w > max_w {
                max_w = w;
            }
        }
        max_w
    }

    pub fn refresh_dynamic(&mut self) {
        for (label, val) in &mut self.lines {
            match label.as_str() {
                "Uptime" => {
                    if let Some(u) = gather_uptime() {
                        *val = u;
                    }
                }
                "Memory" => {
                    if let Some(m) = gather_memory() {
                        *val = m;
                    }
                }
                _ => {}
            }
        }
    }
}

fn gather_os() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                return Some(name.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn gather_host() -> Option<String> {
    let name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").ok()?;
    let trimmed = name.trim();
    if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn gather_kernel() -> Option<String> {
    let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
    if unsafe { libc::uname(&mut uts) } == 0 {
        let release = unsafe { std::ffi::CStr::from_ptr(uts.release.as_ptr()) };
        return Some(release.to_string_lossy().into_owned());
    }
    None
}

pub fn gather_uptime() -> Option<String> {
    let s = fs::read_to_string("/proc/uptime").ok()?;
    let secs_f = s.split_whitespace().next()?.parse::<f64>().ok()?;
    let total_secs = secs_f as u64;

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;

    if days > 0 {
        Some(format!("{days}d {hours}h {mins}m"))
    } else if hours > 0 {
        Some(format!("{hours}h {mins}m"))
    } else {
        Some(format!("{mins}m"))
    }
}

fn gather_packages() -> Option<String> {
    // Check pacman
    if std::path::Path::new("/var/lib/pacman/local").is_dir()
        && let Ok(entries) = fs::read_dir("/var/lib/pacman/local")
    {
        let count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
            .count();
        if count > 0 {
            return Some(format!("{count} (pacman)"));
        }
    }
    // Check dpkg
    if std::path::Path::new("/var/lib/dpkg/status").exists()
        && let Ok(content) = fs::read_to_string("/var/lib/dpkg/status")
    {
        let count = content
            .lines()
            .filter(|l| l.starts_with("Package: "))
            .count();
        if count > 0 {
            return Some(format!("{count} (dpkg)"));
        }
    }
    None
}

fn gather_shell() -> Option<String> {
    let shell = std::env::var("SHELL").ok()?;
    let name = std::path::Path::new(&shell).file_name()?.to_string_lossy();
    Some(name.into_owned())
}

fn gather_wm() -> Option<String> {
    if let Ok(wm) = std::env::var("XDG_CURRENT_DESKTOP")
        && !wm.is_empty()
    {
        return Some(wm);
    }
    if let Ok(wm) = std::env::var("DESKTOP_SESSION")
        && !wm.is_empty()
    {
        return Some(wm);
    }
    None
}

fn gather_terminal() -> Option<String> {
    std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("TERM"))
        .ok()
}

fn gather_cpu() -> Option<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let name = parts[1].trim();
                // Clean common redundant prefixes
                let cleaned = name
                    .replace("(R)", "")
                    .replace("(TM)", "")
                    .replace("CPU", "")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some(cleaned);
            }
        }
    }
    None
}

pub fn gather_memory() -> Option<String> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb: Option<u64> = None;
    let mut avail_kb: Option<u64> = None;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok());
        } else if line.starts_with("MemAvailable:") {
            avail_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok());
        }
        if total_kb.is_some() && avail_kb.is_some() {
            break;
        }
    }

    if let (Some(total), Some(avail)) = (total_kb, avail_kb) {
        let used = total.saturating_sub(avail);
        let used_mib = used / 1024;
        let total_mib = total / 1024;
        let pct = (used as f64 / total as f64 * 100.0) as u32;
        return Some(format!("{used_mib}MiB / {total_mib}MiB ({pct}%)"));
    }
    None
}
