use clap::{Args, Parser, Subcommand};
use course_setup::{pi, typst, vscode};
use std::env;
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let (options, interactive) = match Cli::parse().command {
        Some(Action::Install(options)) => (options, false),
        None => {
            detach_console();
            (InstallOptions { all: true, ..Default::default() }, true)
        }
    };

    let Ok(mut root) = env::current_exe() else {
        let message = "无法定位安装程序所在目录。";
        if interactive {
            dialog(message, false);
        } else {
            eprintln!("{message}");
        }
        return ExitCode::FAILURE;
    };
    root.pop();

    if interactive
        && !dialog(
            "将为当前用户安装课程环境。建议先关闭 VS Code 和 pi。\n\
         确认后请等待结果窗口，不要重复运行。",
            true,
        )
    {
        return ExitCode::SUCCESS;
    }

    let results = install(&root, &options);
    let ok = results.iter().all(|(_, result)| result.is_ok());
    let report: Vec<_> = results
        .into_iter()
        .map(|(name, result)| match result {
            Ok(detail) => format!("{name}：{detail}"),
            Err(err) => format!("{name}：失败（{err}）"),
        })
        .collect();
    let report = report.join("\n\n");
    if interactive {
        dialog(&report, false);
    } else if ok {
        println!("{report}");
    } else {
        eprintln!("{report}");
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn install(root: &Path, options: &InstallOptions) -> Vec<(&'static str, io::Result<String>)> {
    let mut results = Vec::new();
    if options.all || options.vscode {
        let plugins = vscode::cli().and_then(|cmd| vscode::install_vsix(cmd, &root.join("pkgs")));
        results.push(("VS Code 插件", plugins));
        #[cfg(windows)]
        results.push(("VS Code 配置", vscode::install_config(&root.join("Code"))));
    }
    if options.all || options.pi {
        results.push(("pi 扩展", pi::install(&root.join("pi/kong-pi-extension-master.zip"))));
    }
    if options.all || options.typst {
        let templates = typst::package_dir().and_then(|target| {
            typst::install(&root.join("typst/modern-cug-report.typ-master.zip"), &target)
        });
        results.push(("Typst 模板", templates));
    }
    results
}

#[derive(Parser)]
#[command(bin_name = env!("CARGO_BIN_NAME"), version, about = "课程环境安装器")]
struct Cli {
    #[command(subcommand)]
    command: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// 安装指定组件
    Install(InstallOptions),
}

#[derive(Args, Default)]
#[group(required = true, multiple = true)]
struct InstallOptions {
    /// 安装全部组件
    #[arg(long, conflicts_with_all = ["vscode", "pi", "typst"])]
    all: bool,

    /// 安装 VS Code 插件，并在 Windows 上更新用户配置
    #[arg(long)]
    vscode: bool,

    /// 安装 pi 扩展
    #[arg(long)]
    pi: bool,

    /// 安装 Typst 模板
    #[arg(long)]
    typst: bool,
}

fn detach_console() {
    // 双击时只保留安装弹窗，命令行模式则保留控制台。
    #[cfg(windows)]
    // FreeConsole 不接收指针，只分离当前进程。
    unsafe {
        windows_sys::Win32::System::Console::FreeConsole();
    }
}

#[cfg(windows)]
fn dialog(text: &str, confirm: bool) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging as win;
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<_>>();
    let style = if confirm { win::MB_OKCANCEL | win::MB_DEFBUTTON2 } else { win::MB_OK };
    // Both UTF-16 strings live until the dialog closes.
    let answer = unsafe {
        win::MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide("课程环境安装").as_ptr(),
            style,
        )
    };
    answer == win::IDOK
}

#[cfg(not(windows))]
fn dialog(text: &str, confirm: bool) -> bool {
    eprintln!("{text}");
    if !confirm {
        return true;
    }
    eprintln!("继续？[y/N]");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).is_ok() && answer.trim().eq_ignore_ascii_case("y")
}

#[cfg(test)]
#[path = "../tests/cli.rs"]
mod tests;
