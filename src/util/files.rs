use std::path::{Path, PathBuf};
use std::process::Command;

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("FleetManager")
}

pub fn attachments_dir() -> PathBuf {
    data_dir().join("attachments")
}

pub fn attach(src: &str) -> Result<String, String> {
    let p = Path::new(src);
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file")?;
    std::fs::create_dir_all(attachments_dir()).map_err(|e| e.to_string())?;
    let stamp = chrono::Local::now().format("%Y%m%d%H%M%S%3f");
    let dest = attachments_dir().join(format!("{}_{}", stamp, name));
    std::fs::copy(p, &dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

pub fn display_name(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    match name.split_once('_') {
        Some((_, rest)) if !rest.is_empty() => rest.to_string(),
        _ => name,
    }
}

pub fn open_path(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}
