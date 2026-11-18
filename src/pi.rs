use crate::{copy_file, copy_tree, ensure_dir, home, unpack_zip};
use std::io;
use std::path::Path;

pub fn install(path: &Path) -> io::Result<String> {
    let extracted = unpack_zip(path)?;
    let source = extracted.path().join("kong-pi-extension-master");
    let agent = home()?.join(".pi/agent");
    ensure_dir(&agent)?;
    copy_tree(&source.join("extensions"), &agent.join("extensions"), None)?;
    copy_file(&source.join("keybindings.json"), &agent, None)?;
    Ok("已安装".into())
}

#[cfg(windows)]
pub fn install_clipboard(source: &Path) -> io::Result<String> {
    use winreg::{RegKey, enums::*};

    let bin = home()?.join(".pi").join("agent").join("bin");
    copy_clipboard(source, &bin)?;

    let (environment, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags("Environment", KEY_QUERY_VALUE | KEY_SET_VALUE)?;
    if crate::add_to_path(&environment, &bin)? {
        notify_environment_changed();
        return Ok("已安装并加入用户 PATH；请从新终端启动 pi".into());
    }
    Ok("已安装，用户 PATH 无需修改".into())
}

#[cfg(windows)]
fn copy_clipboard(source: &Path, bin: &Path) -> io::Result<()> {
    ensure_dir(bin)?;
    let temp = tempfile::NamedTempFile::new_in(bin)?;
    std::fs::copy(source, temp.path())
        .map_err(|err| io::Error::other(format!("{}：{err}", source.display())))?;
    temp.persist(bin.join("clipboard_image.exe"))?;
    Ok(())
}

#[cfg(windows)]
fn notify_environment_changed() {
    use windows_sys::Win32::UI::WindowsAndMessaging as win;

    let name: Vec<_> = "Environment\0".encode_utf16().collect();
    // SAFETY: 同步发送消息，UTF-16 字符串在调用期间有效。
    unsafe {
        win::SendMessageTimeoutW(
            win::HWND_BROADCAST,
            win::WM_SETTINGCHANGE,
            0,
            name.as_ptr() as isize,
            win::SMTO_ABORTIFHUNG,
            1000,
            std::ptr::null_mut(),
        );
    }
}

#[cfg(all(windows, test))]
#[path = "../tests/pi.rs"]
mod tests;
