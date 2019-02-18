use std::io;

use structopt::{clap::Shell, StructOpt};

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To generate completions for Bash:
    $ deck completion bash > ./deck-completion.bash

    To generate completions for Fish:
    $ deck completion fish > ./deck-completion.fish

    To generate completions for zsh:
    $ deck completion zsh > ./deck-completion.zsh
"#;

#[derive(Debug, StructOpt)]
pub struct Completion {
    #[structopt(subcommand)]
    shell: SupportedShell,
}

#[derive(Debug, StructOpt)]
enum SupportedShell {
    /// Generate completion for Bash
    #[structopt(name = "bash")]
    Bash,
    /// Generate completion for Fish
    #[structopt(name = "fish")]
    Fish,
    /// Generate completion for zsh
    #[structopt(name = "zsh")]
    Zsh,
}

impl From<SupportedShell> for Shell {
    fn from(shell: SupportedShell) -> Self {
        match shell {
            SupportedShell::Bash => Shell::Bash,
            SupportedShell::Fish => Shell::Fish,
            SupportedShell::Zsh => Shell::Zsh,
        }
    }
}

impl CliCommand for Completion {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        let mut app = crate::Opt::clap();
        app.gen_completions_to("deck", self.shell.into(), &mut io::stdout());
        Ok(())
    }
}
