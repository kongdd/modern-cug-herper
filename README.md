# 课程环境安装器

[![CI](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml/badge.svg)](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kongdd/modern-cug-herper/branch/main/graph/badge.svg)](https://codecov.io/gh/kongdd/modern-cug-herper)


- https://github.com/CUG-hydro/modern-cug-report.typ/archive/refs/heads/master.zip
- https://github.com/kongdd/kong-pi-extension/archive/refs/heads/master.zip

**Windows x64：先安装 VS Code，再解压整个安装包，双击 `modern-cug-herper.exe`，点「确定」。**
建议安装前关闭 VS Code 和 pi，安装后重新打开。不需要管理员权限，也不用修改 PowerShell 策略。

安装程序会安装 `pkgs/` 中的 VSIX、pi 扩展及 Typst 模板，并在最后显示每项结果。
Windows 下还会覆盖 `%APPDATA%\Code\User` 中与 `Code/` 同名的文件；旧文件保存在
`%APPDATA%\Code\User.backup-…`，未被覆盖的文件保留在原处。

## 命令行

```text
modern-cug-herper install --all
modern-cug-herper install --vscode
modern-cug-herper install --pi --typst
modern-cug-herper install --help
```

命令行指定组件后直接安装，不弹窗。使用 Windows 分发包时，在 PowerShell 中运行
`.\modern-cug-herper.exe install --all`。双击安装方式不变。

## 构建与分发

维护者在 Windows x64 上准备好 `pkgs/` 中的 VSIX 和 `pi/`、`typst/` 下的安装 ZIP，
运行 `.\build.ps1`，生成 `target/课程环境安装-Windows-x64.zip`。
学生只需解压发布包并双击 `modern-cug-herper.exe`，不用运行 PowerShell 脚本。

Linux/macOS 版在终端运行，需确保 `code` 在 PATH 中；不覆盖 VS Code 用户配置。
当前资源中的 Tinymist VSIX 是 Windows x64 版；给其他平台分发前要换成对应版本。

## Typst 编译测试

```sh
cargo test compiles_typst_with_installed_package -- --ignored
```

需将 `typst` 加入 PATH，并准备随包分发的 Typst ZIP。测试在临时目录安装本地包并编译
`tests/test.typ`；若未缓存 `@preview` 依赖，首次编译需要联网。
