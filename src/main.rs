use clap::{Args, Parser, Subcommand};
use course_setup::{pi, typst, vscode};
use std::env;
use std::io;
use std::path::Path;
use std::process::ExitCode;

#[cfg(windows)]
mod winui;

const RELOAD_HINT: &str = "若 VS Code 已打开，请执行 Developer: Reload Window。";

fn main() -> ExitCode {
    let (options, interactive) = match Cli::parse().command {
        Some(Action::Install(options)) => (options, false),
        None => {
            detach_console();
            (InstallOptions { all: true, ..Default::default() }, true)
        }
    };

    #[cfg(windows)]
    let _lock = match course_setup::lock_installation() {
        Ok(lock) => lock,
        Err(err) => return fail(&format!("无法启动安装：{err}"), interactive),
    };

    let Ok(mut root) = env::current_exe() else {
        return fail("无法定位安装程序所在目录。", interactive);
    };
    root.pop();

    if interactive && !dialog("将为当前用户安装课程环境。", true) {
        return ExitCode::SUCCESS;
    }

    let steps = steps(&options);
    #[cfg(windows)]
    if interactive {
        return match winui::run(root, steps) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(err) => fail(&format!("安装窗口出错：{err}"), true),
        };
    }

    let results = install(&root, steps, |text| eprintln!("{text}"));
    let (ok, report) = summary(results, options.all || options.vscode);
    if interactive {
        dialog(&report, false);
    } else if ok {
        println!("{report}");
    } else {
        eprintln!("{report}");
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn summary(results: Vec<(&str, io::Result<String>)>, reload_hint: bool) -> (bool, String) {
    let ok = results.iter().all(|(_, result)| result.is_ok());
    let report: Vec<_> = results
        .into_iter()
        .map(|(name, result)| match result {
            Ok(detail) => format!("{name}：{detail}"),
            Err(err) => format!("{name}：失败（{err}）"),
        })
        .collect();
    let mut report = report.join("\n");
    if reload_hint {
        report.push('\n');
        report.push_str(RELOAD_HINT);
    }
    (ok, report)
}

fn fail(message: &str, interactive: bool) -> ExitCode {
    if interactive {
        dialog(message, false);
    } else {
        eprintln!("{message}");
    }
    ExitCode::FAILURE
}

type Step = (&'static str, fn(&Path) -> io::Result<String>);

fn steps(options: &InstallOptions) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    if options.all || options.vscode {
        steps.push(("VS Code 插件", |root| {
            vscode::cli().and_then(|cmd| vscode::install_vsix(cmd, &root.join("pkgs")))
        }));
        #[cfg(windows)]
        steps.push(("VS Code 配置", |root| vscode::install_config(&root.join("Code"))));
    }
    if options.all || options.pi {
        steps
            .push(("pi 扩展", |root| pi::install(&root.join("pi/kong-pi-extension-master.zip"))));
        #[cfg(windows)]
        steps.push(("pi 剪贴板工具", |root| {
            pi::install_clipboard(&root.join("pi/clipboard_image.exe"))
        }));
    }
    if options.all || options.typst {
        steps.push(("Typst 模板", |root| {
            typst::install(
                &root.join("typst/modern-cug-report.typ-master.zip"),
                &typst::package_dir()?,
            )
        }));
    }

    steps
}

fn install(
    root: &Path,
    steps: Vec<Step>,
    mut show_progress: impl FnMut(String),
) -> Vec<(&'static str, io::Result<String>)> {
    let total = steps.len();
    steps
        .into_iter()
        .enumerate()
        .map(|(index, (name, run))| {
            show_progress(format!("[{}/{total}] 正在安装 {name}…", index + 1));
            (name, run(root))
        })
        .collect()
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
    // 双击时只显示安装窗口，命令行模式仍使用控制台。
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
    // SAFETY: 两个 UTF-16 字符串在调用期间有效。
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
