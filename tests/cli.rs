use crate::{Action, Cli};
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
