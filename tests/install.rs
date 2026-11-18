use crate::vscode::{self, install_vsix};
use crate::{copy_tree, typst, unpack_zip};
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

#[test]
fn backs_up_only_replaced_files() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;
    let src = tmp.path().join("source");
    let dst = tmp.path().join("target");
    let backup = tmp.path().join("backup");
    fs::create_dir_all(src.join("snippets"))?;
    fs::create_dir_all(dst.join("snippets"))?;
    fs::write(src.join("snippets/course.json"), "new")?;
    fs::write(dst.join("snippets/course.json"), "old")?;
    fs::write(dst.join("unrelated.json"), "keep")?;
    copy_tree(&src, &dst, Some(&backup))?;
    assert_eq!(fs::read_to_string(backup.join("snippets/course.json"))?, "old");
    assert_eq!(fs::read_to_string(dst.join("snippets/course.json"))?, "new");
    assert_eq!(fs::read_to_string(dst.join("unrelated.json"))?, "keep");
    assert!(!backup.join("unrelated.json").exists());
    Ok(())
}

#[test]
fn rejects_archive_traversal() -> io::Result<()> {
    use std::io::Write;
    let tmp = tempfile::tempdir()?;
    let path = tmp.path().join("input.zip");
    let mut zip = zip::ZipWriter::new(File::create(&path)?);
    zip.start_file("../outside.txt", zip::write::SimpleFileOptions::default())?;
    zip.write_all(b"outside")?;
    zip.finish()?;
    assert!(unpack_zip(&path).is_err());
    Ok(())
}

#[test]
#[ignore = "requires the bundled pi archive"]
fn unpacks_bundled_pi_extensions() -> io::Result<()> {
    let pi = unpack_zip(Path::new("pi/kong-pi-extension-master.zip"))?;
    assert!(pi.path().join("kong-pi-extension-master/extensions/Friday/index.ts").is_file());
    Ok(())
}

#[test]
#[ignore = "requires Typst CLI and the bundled Typst archive"]
fn compiles_typst_with_installed_package() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let packages = sandbox.path().join("packages");
    let target = packages.join("local/modern-cug-report");
    typst::install(Path::new("typst/modern-cug-report.typ-master.zip"), &target)?;
    assert!(!target.join("README.md").exists());

    let pdf = sandbox.path().join("test.pdf");
    let output = Command::new("typst")
        .args(["compile", "--package-path"])
        .arg(&packages)
        .arg("tests/test.typ")
        .arg(&pdf)
        .output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(pdf.is_file());
    Ok(())
}

#[test]
#[ignore = "requires VS Code and bundled VSIX"]
fn installs_multiple_vsix_in_sandbox() -> io::Result<()> {
    let sandbox = tempfile::tempdir()?;
    let pkgs = sandbox.path().join("测试 packages");
    fs::create_dir(&pkgs)?;
    fs::copy("pkgs/kongdd.typst-title-numering-0.1.2.vsix", pkgs.join("first test.vsix"))?;
    fs::copy("pkgs/zenyui.vscode-pi-bridge-0.0.3.vsix", pkgs.join("second test.vsix"))?;
    let extensions = sandbox.path().join("extensions");
    let cli = || {
        let mut cli = vscode::cli()?;
        cli.arg("--extensions-dir")
            .arg(&extensions)
            .arg("--user-data-dir")
            .arg(sandbox.path().join("user-data"));
        Ok(cli)
    };
    install_vsix(cli, &pkgs, |_, _, _| {})?;
    let count =
        fs::read_dir(extensions)?.filter(|e| e.as_ref().is_ok_and(|e| e.path().is_dir())).count();
    assert_eq!(count, 2);
    Ok(())
}
