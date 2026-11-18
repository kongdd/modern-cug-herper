## vscode

- 安装pkgs文件夹下的所有vsix插件

- 将Code文件夹下的所有内容覆盖到用户`${env:APPDATA}/Code/User`（仅针对Windows系统）

## pi

将`pi/kong-pi-extension-master.zip`下的extensions插件，全部安装到`~/.pi/agent/extensions/`，如果不存在该目录，则创建之。

## typst

将`typst/modern-cug-report.typ-master.zip`安装到用户目录：

- windows: `$env:APPDATA/typst/packages/local`
- linux  : `~/.local/share/typst/packages/local`
- mac    : `~/Library/Application Support/typst/packages/local`
