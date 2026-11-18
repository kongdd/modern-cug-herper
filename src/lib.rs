pub mod pi;
pub mod typst;
pub mod vscode;

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::read::{ExtractOptions, SymlinkExtractAction};

pub(crate) fn user_dir(var: &str) -> io::Result<PathBuf> {
    env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| io::Error::other(format!("找不到用户目录：{var}")))
}

#[cfg(windows)]
pub fn lock_installation() -> io::Result<File> {
    use std::fs::{OpenOptions, TryLockError};

    let dir = user_dir("LOCALAPPDATA")?;
    fs::create_dir_all(&dir)?;
    // 沿用旧文件名，与旧版互斥；只锁不删，避免另一个进程锁住同名的新文件。
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("modern-cug-herper.lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "已有安装程序正在运行，请等待完成。"))
        }
        Err(TryLockError::Error(err)) => Err(err),
    }
}

#[cfg(windows)]
pub(crate) fn add_to_path(environment: &winreg::RegKey, dir: &Path) -> io::Result<bool> {
    use std::ffi::{OsStr, OsString};
    use winreg::enums::{REG_EXPAND_SZ, REG_SZ};
    use winreg::types::{FromRegValue, ToRegValue};

    let contains = |path: &OsStr| {
        env::split_paths(path).any(|entry| {
            entry
                .iter()
                .map(OsStr::to_ascii_lowercase)
                .eq(dir.iter().map(OsStr::to_ascii_lowercase))
        })
    };
    if env::var_os("PATH").is_some_and(|path| contains(&path)) {
        return Ok(false);
    }

    let (mut path, kind) = match environment.get_raw_value("Path") {
        Ok(value) if matches!(value.vtype, REG_SZ | REG_EXPAND_SZ) => {
            (OsString::from_reg_value(&value)?, value.vtype)
        }
        Ok(_) => return Err(io::Error::other("用户 PATH 不是字符串，未修改")),
        Err(err) if err.kind() == io::ErrorKind::NotFound => (OsString::new(), REG_SZ),
        Err(err) => return Err(err),
    };
    if contains(&path) {
        return Ok(false);
    }

    // 保留原有条目和 REG_EXPAND_SZ 类型，不展开其中的变量。
    if !path.is_empty() && !path.as_encoded_bytes().ends_with(b";") {
        path.push(";");
    }
    path.push(env::join_paths([dir]).map_err(io::Error::other)?);
    let mut value = path.to_reg_value();
    value.vtype = kind;
    environment.set_raw_value("Path", &value)?;
    Ok(true)
}

pub(crate) fn ensure_dir(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|meta| !meta.is_dir()) {
        return Err(io::Error::other(format!("不是普通目录：{}", path.display())));
    }
    fs::create_dir_all(path)
}

pub(crate) fn copy_tree(
    source: &Path,
    destination: &Path,
    backup_dir: Option<&Path>,
) -> io::Result<()> {
    let entries = fs::read_dir(source)?;
    ensure_dir(destination)?;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let source_path = entry.path();
        let target = destination.join(&name);
        if entry.file_type()?.is_dir() {
            let backup_subdir = backup_dir.map(|dir| dir.join(&name));
            copy_tree(&source_path, &target, backup_subdir.as_deref())?;
            continue;
        }
        copy_file(&source_path, destination, backup_dir)?;
    }
    Ok(())
}

pub(crate) fn copy_file(
    source: &Path,
    destination: &Path,
    backup_dir: Option<&Path>,
) -> io::Result<()> {
    if !fs::symlink_metadata(source)?.is_file() {
        return Err(io::Error::other(format!("不是普通文件：{}", source.display())));
    }
    let name = source.file_name().unwrap();
    let target = destination.join(name);
    match fs::symlink_metadata(&target) {
        Ok(meta) if meta.is_file() => {
            if let Some(dir) = backup_dir {
                fs::create_dir_all(dir)?;
                fs::copy(&target, dir.join(name))?;
            }
        }
        Ok(_) => return Err(io::Error::other(format!("不能覆盖：{}", target.display()))),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }

    // 复制成功后才替换目标，避免留下只写了一部分的文件。
    let temp = tempfile::NamedTempFile::new_in(destination)?;
    fs::copy(source, temp.path())?;
    temp.persist(target)?;
    Ok(())
}

pub(crate) fn unpack_zip(path: &Path) -> io::Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let options = ExtractOptions::default().symlink_action(SymlinkExtractAction::NoExtract);
    zip::ZipArchive::new(File::open(path)?)?.extract_with_options(dir.path(), options)?;
    Ok(dir)
}

pub(crate) fn home() -> io::Result<PathBuf> {
    user_dir(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
}

#[cfg(test)]
#[path = "../tests/install.rs"]
mod tests;
