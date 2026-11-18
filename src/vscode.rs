#[cfg(windows)]
use crate::{copy_tree, user_dir};
#[cfg(windows)]
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
pub fn install_config(src: &Path) -> io::Result<String> {
    let code = user_dir("APPDATA")?.join("Code");
    let target = code.join("User");
    if !target.is_dir() {
        copy_tree(src, &target, None)?;
        return Ok("已安装".into());
    }

    let backup = tempfile::Builder::new().prefix("User.backup-").tempdir_in(code)?.keep();
    copy_tree(src, &target, Some(&backup))
        .map_err(|err| io::Error::other(format!("{err}\n备份：{}", backup.display())))?;
    Ok(format!("已安装，备份：{}", backup.display()))
}

#[cfg(windows)]
pub fn cli() -> io::Result<Command> {
    use std::os::windows::process::CommandExt;

    let default_locations = [
        ("LOCALAPPDATA", "Programs/Microsoft VS Code/bin"),
        ("ProgramFiles", "Microsoft VS Code/bin"),
    ];
    let default_bins = default_locations
        .into_iter()
        .filter_map(|(var, subdir)| user_dir(var).ok().map(|dir| dir.join(subdir)));
    let path = env::var_os("PATH").unwrap_or_default();
    let path_bins = env::split_paths(&path).filter(|dir| dir.is_absolute());
    let cli = default_bins
        .chain(path_bins)
        .map(|dir| dir.join("code.cmd"))
        .find(|file| file.is_file())
        .ok_or_else(|| io::Error::other("找不到 VS Code，请先安装。"))?;

    let mut command = Command::new(cli);
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    Ok(command)
}

#[cfg(not(windows))]
pub fn cli() -> io::Result<Command> {
    Ok(Command::new("code"))
}

pub fn install_vsix(mut command: Command, dir: &Path) -> io::Result<String> {
    let mut count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let is_vsix = entry.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("vsix"));
        if !entry.file_type()?.is_file() || !is_vsix {
            continue;
        }
        command.arg("--install-extension").arg(Path::new(".").join(entry.file_name()));
        count += 1;
    }
    if count == 0 {
        return Err(io::Error::other("pkgs 中没有 VSIX 文件"));
    }
    let output = command.current_dir(dir).output()?;
    if !output.status.success() {
        let detail = [output.stderr, output.stdout].concat();
        let detail: String = String::from_utf8_lossy(&detail).trim().chars().take(750).collect();
        return Err(io::Error::other(format!("{}\n{detail}", output.status)));
    }
    Ok(format!("已安装 {count} 个"))
}
