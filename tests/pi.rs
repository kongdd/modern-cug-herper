use super::*;
use std::ffi::OsString;
use std::fs;
use winreg::{RegKey, enums::*, types::ToRegValue};

#[test]
fn installs_clipboard_and_preserves_user_path() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let source = sandbox.path().join("clipboard_image.exe");
    let bin = sandbox.path().join("home/.pi/agent/bin");
    fs::write(&source, b"clipboard helper")?;
    copy_clipboard(&source, &bin)?;

    // 使用临时注册表项，不写真实的 HKCU\Environment。
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let name = format!(
        "Software\\modern-cug-herper-test-{}",
        sandbox.path().file_name().unwrap().display()
    );
    let (key, _) = user.create_subkey(&name)?;
    let result = (|| -> io::Result<_> {
        let original = r"%LOCALAPPDATA%\Tools;C:\Other";
        let mut value = original.to_reg_value();
        value.vtype = REG_EXPAND_SZ;
        key.set_raw_value("Path", &value)?;
        let process_path = std::env::var_os("PATH").unwrap_or_default();
        if let Some(entry) = std::env::split_paths(&process_path).find(|dir| dir.is_absolute()) {
            let skipped = !crate::add_to_path(&key, &entry)?;
            if !skipped {
                return Err(io::Error::other("不应重复追加进程 PATH 中的目录"));
            }
        }
        let added = crate::add_to_path(&key, &bin)?;
        let added_again = crate::add_to_path(&key, &bin.join(""))?;
        let path: OsString = key.get_value("Path")?;
        let value = key.get_raw_value("Path")?;
        Ok((added, added_again, path, value.vtype))
    })();
    drop(key);
    user.delete_subkey_all(name)?;

    let (added, added_again, path, kind) = result?;
    assert!(added && !added_again);
    assert_eq!(kind, REG_EXPAND_SZ);
    let paths: Vec<_> = std::env::split_paths(&path).collect();
    assert_eq!(paths, [r"%LOCALAPPDATA%\Tools".into(), r"C:\Other".into(), bin.clone()]);
    assert_eq!(fs::read(bin.join("clipboard_image.exe"))?, b"clipboard helper");
    Ok(())
}
