# 维护者测试指南

## Rust 覆盖率

使用 Windows MSVC 工具链（需安装 `llvm-tools-preview` 和 `cargo-llvm-cov`）运行：

```sh
cargo +stable-msvc llvm-cov --show-missing-lines
```

安装器集成测试在临时目录运行，无需下载 VSIX 或 ZIP，也不会调用本机 VS Code。

## VS Code 片段测试

在 Windows x64 上准备好 Node.js 22+ 和 `pkgs/` 中的 Tinymist VSIX，从仓库根目录运行：

```sh
cd tests/vscode
npm install --no-package-lock
npm test
```

首次运行会下载 VS Code。测试在临时配置中验证 Typst 片段的展开和光标位置，
不会修改日常使用的 VS Code 配置。GitHub Actions 也会运行此测试。

## Typst 编译测试

从仓库根目录运行：

```sh
cargo test compiles_typst_with_installed_package -- --ignored
```

需将 `typst` 加入 PATH，并准备随包分发的 Typst ZIP。测试在临时目录安装本地包并编译
`tests/test.typ`；若未缓存 `@preview` 依赖，首次编译需要联网。
