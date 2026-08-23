use std::env;
use std::sync::OnceLock;

pub const RESET: &str = "\u{1b}[0m";
pub const DIM: &str = "\u{1b}[2m";
pub const BOLD: &str = "\u{1b}[1m";
pub const GREEN: &str = "\u{1b}[32m";
pub const RED: &str = "\u{1b}[31m";
pub const YELLOW: &str = "\u{1b}[33m";
pub const ON_GREEN: &str = "\u{1b}[42;30;1m";
pub const ON_RED: &str = "\u{1b}[41;97;1m";

pub fn plain() -> bool {
    static PLAIN: OnceLock<bool> = OnceLock::new();

    *PLAIN.get_or_init(|| env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()))
}

pub fn paint(style: &str, text: &str) -> String {
    match plain() {
        true => text.to_owned(),
        false => format!("{style}{text}{RESET}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_painted_word_carries_the_style_and_the_reset() {
        if plain() {
            return;
        }

        let painted = paint(GREEN, "pass");

        assert!(painted.starts_with(GREEN));
        assert!(painted.ends_with(RESET));
        assert!(painted.contains("pass"));
    }
}
