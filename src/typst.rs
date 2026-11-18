use crate::{copy_tree, ensure_dir, home, unpack_zip, user_dir};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn package_dir() -> io::Result<PathBuf> {
    let data = if cfg!(windows) {
        user_dir("APPDATA")?
    } else if cfg!(target_os = "macos") {
        home()?.join("Library/Application Support")
    } else {
        user_dir("XDG_DATA_HOME").or_else(|_| home().map(|p| p.join(".local/share")))?
    };
    Ok(data.join("typst/packages/local/modern-cug-report"))
}

pub fn install(path: &Path, target: &Path) -> io::Result<String> {
    let extracted = unpack_zip(path)?;
    let package = extracted.path().join("modern-cug-report.typ-master");
    ensure_dir(target)?;

    let mut count = 0;
    for entry in fs::read_dir(package)? {
        let entry = entry?;
        let version_dir = entry.path();
        if !version_dir.join("typst.toml").is_file() {
            continue;
        }
        copy_tree(&version_dir, &target.join(entry.file_name()), None)?;
        count += 1;
    }
    if count == 0 {
        return Err(io::Error::other("找不到带 typst.toml 的版本目录"));
    }
    Ok(format!("已安装 {count} 个版本"))
}
