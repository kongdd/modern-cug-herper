#![cfg(windows)]

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const CODE_CLI: &str = "local/Programs/Microsoft VS Code/bin/code.cmd";

fn installer(root: &Path) -> io::Result<Command> {
    let exe = root.join("installer.exe");
    fs::copy(env!("CARGO_BIN_EXE_modern-cug-herper"), &exe)?;
    let mut cmd = Command::new(exe);
    // 只为子进程设置环境变量，不使用本机的用户目录或 VS Code。
    cmd.env("APPDATA", root.join("appdata"))
        .env("USERPROFILE", root.join("home"))
        .env("LOCALAPPDATA", root.join("local"))
        .env("PATH", root)
        .arg("install");
    Ok(cmd)
}

fn write(path: &Path, contents: &str) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, contents)
}

fn write_zip(path: &Path, files: &[(&str, &str)]) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut zip = zip::ZipWriter::new(File::create(path)?);
    for &(name, contents) in files {
        zip.start_file(name, zip::write::SimpleFileOptions::default())?;
        zip.write_all(contents.as_bytes())?;
    }
    zip.finish()?;
    Ok(())
}

#[test]
fn installs_vscode_and_typst_and_backs_up_settings() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    let mut cmd = installer(root)?;
    write(
        &root.join(CODE_CLI),
        concat!(
            "@echo off\r\n",
            "if not \"%~1\"==\"--install-extension\" exit /b 1\r\n",
            "if not \"%~3\"==\"--install-extension\" exit /b 1\r\n",
            "if not exist \"%~2\" exit /b 1\r\n",
            "if not exist \"%~4\" exit /b 1\r\n",
            "exit /b 0\r\n",
        ),
    )?;
    write(&root.join("pkgs/first extension.vsix"), "")?;
    write(&root.join("pkgs/second extension.VSIX"), "")?;
    write(&root.join("pkgs/README.txt"), "ignored")?;
    write(&root.join("Code/settings.json"), "{}")?;
    write_zip(
        &root.join("typst/modern-cug-report.typ-master.zip"),
        &[
            ("modern-cug-report.typ-master/1.0.0/typst.toml", "[package]"),
            ("modern-cug-report.typ-master/README.md", "ignored"),
        ],
    )?;

    let output = cmd.args(["--vscode", "--typst"]).output()?;
    let progress = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{progress}");
    assert_eq!(
        progress.lines().collect::<Vec<_>>(),
        [
            "[1/3] 正在安装 VS Code 插件…",
            "[2/3] 正在安装 VS Code 配置…",
            "[3/3] 正在安装 Typst 模板…"
        ]
    );
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.contains("VS Code 插件：已安装 2 个"), "{report}");
    assert!(report.contains("Typst 模板：已安装 1 个版本"), "{report}");

    let code = root.join("appdata/Code");
    let settings = code.join("User/settings.json");
    assert_eq!(fs::read_to_string(&settings)?, "{}");
    let typst = root.join("appdata/typst/packages/local/modern-cug-report");
    assert_eq!(fs::read_to_string(typst.join("1.0.0/typst.toml"))?, "[package]");
    assert!(!typst.join("README.md").exists());

    fs::write(&settings, r#"{"editor.fontSize": 18}"#)?;
    let output = cmd.output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(settings)?, "{}");
    let report = String::from_utf8_lossy(&output.stdout);
    let backup = report
        .lines()
        .find_map(|line| line.strip_prefix("VS Code 配置：已安装，备份："))
        .expect("expected backup path in the installation report");
    let backup = Path::new(backup);
    assert_eq!(backup.parent(), Some(code.as_path()));
    assert_eq!(fs::read_to_string(backup.join("settings.json"))?, r#"{"editor.fontSize": 18}"#);
    Ok(())
}

#[test]
fn reports_missing_vsix_and_invalid_appdata() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    write(&root.join(CODE_CLI), "exit /b 1\r\n")?;
    fs::create_dir(root.join("pkgs"))?;
    let output =
        installer(root)?.env("APPDATA", "relative").args(["--vscode", "--typst"]).output()?;
    let report = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("VS Code 插件：失败（pkgs 中没有 VSIX 文件"), "{report}");
    assert!(report.contains("VS Code 配置：失败（找不到用户目录：APPDATA"), "{report}");
    assert!(report.contains("Typst 模板：失败（找不到用户目录：APPDATA"), "{report}");
    assert!(!report.contains("pi 扩展"), "{report}");
    Ok(())
}

#[test]
fn reports_vsix_failure_without_skipping_settings() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    write(&root.join(CODE_CLI), "@echo off\r\necho VSIX rejected 1>&2\r\nexit /b 1\r\n")?;
    write(&root.join("pkgs/test.vsix"), "")?;
    write(&root.join("Code/settings.json"), "{}")?;

    let output = installer(root)?.arg("--vscode").output()?;
    let report = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(
        report.contains("VS Code 插件：失败（") && report.contains("VSIX rejected"),
        "{report}"
    );
    assert!(report.contains("VS Code 配置：已安装"), "{report}");
    assert_eq!(fs::read_to_string(root.join("appdata/Code/User/settings.json"))?, "{}");
    Ok(())
}

#[test]
fn installs_pi_extensions_even_if_clipboard_helper_is_missing() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    write_zip(
        &root.join("pi/kong-pi-extension-master.zip"),
        &[("kong-pi-extension-master/extensions/Friday/index.ts", "extension")],
    )?;

    let output = installer(root)?.arg("--pi").output()?;
    let report = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("pi 扩展：已安装"), "{report}");
    assert!(report.contains("pi 剪贴板工具：失败") && report.contains("clipboard_image.exe"));
    assert_eq!(
        fs::read_to_string(root.join("home/.pi/agent/extensions/Friday/index.ts"))?,
        "extension"
    );
    Ok(())
}

#[test]
fn blocks_installation_while_lock_is_held() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    fs::create_dir(root.join("local"))?;
    let lock = File::create_new(root.join("local/modern-cug-herper.lock"))?;
    lock.try_lock()?;
    write_zip(
        &root.join("typst/modern-cug-report.typ-master.zip"),
        &[("modern-cug-report.typ-master/1.0.0/typst.toml", "[package]")],
    )?;

    let mut cmd = installer(root)?;
    let output = cmd.arg("--typst").output()?;
    let report = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("已有安装程序正在运行"), "{report}");
    assert!(!root.join("appdata").exists());
    drop(lock);
    let output = cmd.output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

#[test]
fn rejects_typst_archive_without_a_version_manifest() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    write_zip(
        &root.join("typst/modern-cug-report.typ-master.zip"),
        &[("modern-cug-report.typ-master/1.0.0/lib.typ", "")],
    )?;

    let output = installer(root)?.arg("--typst").output()?;
    let report = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report.contains("Typst 模板：失败（找不到带 typst.toml 的版本目录"), "{report}");
    Ok(())
}
