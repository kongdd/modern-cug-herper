# 课程环境安装器

[![CI](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml/badge.svg)](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kongdd/modern-cug-herper/branch/master/graph/badge.svg)](https://codecov.io/gh/kongdd/modern-cug-herper)


- https://github.com/CUG-hydro/modern-cug-report.typ/archive/refs/heads/master.zip
- https://github.com/kongdd/kong-pi-extension/archive/refs/heads/master.zip

**Windows x64：先安装 VS Code，再解压整个安装包，双击 `modern-cug-herper.exe`，点「确定」。**
建议安装前关闭 VS Code，安装后重新打开。安装后在 pi 中执行 `/reload` 即可加载扩展。

安装期间，控制台会显示当前步骤（如 `[1/5] 正在安装 VS Code 插件…`），结束后弹窗显示各项结果。
同一 Windows 用户不能同时运行两个安装实例，重复启动会提示退出。
安装程序会安装 `pkgs/` 中的 VSIX、pi 扩展及 Typst 模板。
Windows 下还会将 `clipboard_image.exe` 安装到 `~/.pi/bin`；仅当当前或已保存的用户 PATH 均未包含该目录时，才追加到用户 PATH。
已运行的 pi 不会继承新的 PATH；要使用剪贴板工具，请从新终端启动 pi。
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
