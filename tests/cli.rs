use crate::{Action, Cli, summary};

#[test]
fn mixed_summary_keeps_success_details_and_errors() {
    let results = vec![
        ("VS Code 插件", Ok("已安装 8 个".into())),
        ("VS Code 配置", Ok("已安装，备份：User.backup-test".into())),
        ("Typst 模板", Err(std::io::Error::other("缺少模板"))),
    ];
    let (ok, report) = summary(results, true);
    assert!(!ok);
    assert_eq!(
        report,
        concat!(
            "VS Code 插件：已安装 8 个\nVS Code 配置：已安装，备份：User.backup-test\n",
            "Typst 模板：失败（缺少模板）\n若 VS Code 已打开，请执行 Developer: Reload Window。"
        )
    );
}
use clap::Parser;

#[test]
fn parses_install_options() {
    let exe = env!("CARGO_BIN_NAME");
    let args = Cli::try_parse_from([exe, "install", "--all"]).unwrap();
    let Some(Action::Install(options)) = args.command else {
        panic!("expected install subcommand");
    };
    assert!(options.all);

    let args = Cli::try_parse_from([exe, "install", "--pi", "--typst"]).unwrap();
    let Some(Action::Install(options)) = args.command else {
        panic!("expected install subcommand");
    };
    assert!(options.pi && options.typst && !options.vscode && !options.all);
    assert!(Cli::try_parse_from([exe, "install"]).is_err());
    assert!(Cli::try_parse_from([exe, "install", "--all", "--pi"]).is_err());
    assert!(Cli::try_parse_from([exe]).unwrap().command.is_none());
}
