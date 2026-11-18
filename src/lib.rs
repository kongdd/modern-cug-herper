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
        let kind = entry.file_type()?;

        if kind.is_dir() {
            let backup_subdir = backup_dir.map(|dir| dir.join(&name));
            copy_tree(&source_path, &target, backup_subdir.as_deref())?;
            continue;
        }
        if !kind.is_file() {
            return Err(io::Error::other(format!("不是普通文件：{}", source_path.display())));
        }

        match fs::symlink_metadata(&target) {
            Ok(meta) if meta.is_file() => {
                if let Some(dir) = backup_dir {
                    fs::create_dir_all(dir)?;
                    fs::copy(&target, dir.join(&name))?;
                }
            }
            Ok(_) => return Err(io::Error::other(format!("不能覆盖：{}", target.display()))),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }

        // 复制成功后才替换目标，避免留下只写了一部分的文件。
        let temp = tempfile::NamedTempFile::new_in(destination)?;
        fs::copy(source_path, temp.path())?;
        temp.persist(target)?;
    }
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
