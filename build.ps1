#Requires -Version 5.1
$ErrorActionPreference = 'Stop'
$zip = 'target/课程环境安装-Windows-x64.zip'
$temp = "target/package-$PID.zip"

Push-Location -LiteralPath $PSScriptRoot
try {
    $target = (rustc -vV | Select-String '^host: x86_64-pc-windows-(gnu|msvc)$') -replace '^host: '
    if (!$target) { throw '请使用 Windows x64 Rust 工具链。' }
    $vsix = Get-ChildItem pkgs -File -Filter '*.vsix' | ForEach-Object { "pkgs/$($_.Name)" }
    if (!$vsix) { throw 'pkgs 中没有 VSIX 文件。' }

    cargo build --release --target $target --target-dir target
    if ($LASTEXITCODE) { throw '构建失败。' }
    Copy-Item -LiteralPath "target/$target/release/modern-cug-herper.exe" -Destination . -Force

    tar.exe -a -cf $temp --options zip:hdrcharset=UTF-8 Code README.md @vsix `
        pi/kong-pi-extension-master.zip pi/clipboard_image.exe `
        typst/modern-cug-report.typ-master.zip `
        -C "target/$target/release" modern-cug-herper.exe
    if ($LASTEXITCODE) { throw '打包失败。' }
    Move-Item -LiteralPath $temp -Destination $zip -Force
    Write-Host $zip
} finally {
    if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp }
    Pop-Location
}
