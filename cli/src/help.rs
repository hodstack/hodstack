use clap::builder::styling::{AnsiColor, Color, Style, Styles};

const BOLD: Style = Style::new().bold();

const DIM: Style = Style::new().dimmed();

pub const STYLES: Styles = Styles::styled()
    .header(BOLD)
    .usage(BOLD)
    .literal(BOLD)
    .placeholder(DIM)
    .error(
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::Red)))
            .bold(),
    )
    .invalid(
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
            .bold(),
    )
    .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .context(DIM);

pub fn template() -> String {
    format!(
        "\n\
         \n\
         {BOLD}{{name}}{BOLD:#} {DIM}{{version}} — {{about}}{DIM:#}\n\
         \n\
         {BOLD}USAGE{BOLD:#}\n\
         \x20\x20{{usage}}\n\
         \n\
         {BOLD}COMMANDS{BOLD:#}\n\
         {{subcommands}}\n\
         \n\
         {BOLD}OPTIONS{BOLD:#}\n\
         {{options}}\n\
         \n\
         {DIM}github.com/hodstack/hodstack{DIM:#}"
    )
}
