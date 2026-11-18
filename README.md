# 课程环境安装器

[![CI](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml/badge.svg)](https://github.com/kongdd/modern-cug-herper/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kongdd/modern-cug-herper/branch/master/graph/badge.svg)](https://codecov.io/gh/kongdd/modern-cug-herper)


- https://github.com/CUG-hydro/modern-cug-report.typ/archive/refs/heads/master.zip
- https://github.com/kongdd/kong-pi-extension/archive/refs/heads/master.zip

**Windows x64：先安装 VS Code，再解压整个安装包，双击 `modern-cug-helper.exe`，点「确定」。**
安装无需关闭 VS Code。安装后，在 VS Code 命令面板执行 `Developer: Reload Window`。
安装后在 pi 中执行 `/reload` 即可加载扩展。

双击安装时，同一窗口显示当前步骤及组件名称。结束后保留逐项汇总，包括安装数量、配置备份路径和失败原因，点击「确定」后关闭。
同一 Windows 用户不能同时运行两个安装实例，重复启动会提示退出。
安装程序会安装 `pkgs/` 中的 VSIX、pi 扩展及 Typst 模板。
安装 pi 扩展时，会用扩展 ZIP 中的 `keybindings.json` 覆盖 `~/.pi/agent/keybindings.json`。
Windows 下还会将 `clipboard_image.exe` 安装到 `~/.pi/agent/bin`（不存在则自动创建）；仅当当前或已保存的用户 PATH 均未包含该目录时，才追加到用户 PATH。
已运行的 pi 不会继承新的 PATH；要使用剪贴板工具，请从新终端启动 pi。
Windows 下还会覆盖 `%APPDATA%\Code\User` 中与 `Code/` 同名的文件；旧文件保存在
`%APPDATA%\Code\User.backup-…`，未被覆盖的文件保留在原处。

## 命令行

```text
modern-cug-helper install --all
modern-cug-helper install --vscode
modern-cug-helper install --pi --typst
modern-cug-helper install --help
```

命令行指定组件后直接安装，在控制台显示进度，不弹窗。使用 Windows 分发包时，在 PowerShell 中运行
`.\modern-cug-helper.exe install --all`。双击安装方式不变。
