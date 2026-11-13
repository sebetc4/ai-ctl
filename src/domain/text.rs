//! Text that comes from outside ai-ctl: the characters it must not carry as
//! they are into a terminal or into a file another program parses; and the
//! message of several errors.

use std::fmt::Display;

/// Whether `c` is a control character, which can drive a terminal, or
/// U+2028 or U+2029, which YAML parsers read as line ends.
pub fn is_control_or_separator(c: char) -> bool {
    c.is_control() || matches!(c, '\u{2028}' | '\u{2029}')
}

/// `c` written as a `\uXXXX` escape.
pub fn unicode_escape(c: char) -> String {
    format!("\\u{:04X}", u32::from(c))
}

/// Whether `c` is a Unicode format character that changes how a terminal
/// shows the text around it while showing nothing itself: the
/// bidirectional marks, embeddings, overrides and isolates, the zero-width
/// characters and the byte order mark. The standard library has no lookup
/// of Unicode's categories, so the list is fixed.
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{61c}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
    )
}

/// `text` fit for a terminal: each control character, line separator and
/// invisible format character written as its `\uXXXX` escape, the rest
/// unchanged.
pub fn escape_controls(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if is_control_or_separator(c) || is_invisible_format(c) {
            escaped.push_str(&unicode_escape(c));
        } else {
            escaped.push(c);
        }
    }
    escaped
}

/// `errors` as one message, each after the other, in their order.
pub fn joined<E: Display>(errors: &[E]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_and_line_separators_are_told_apart_from_text() {
        for c in [
            '\u{1b}', '\n', '\0', '\u{7f}', '\u{9b}', '\u{2028}', '\u{2029}',
        ] {
            assert!(is_control_or_separator(c), "{c:?}");
        }
        for c in ['a', ' ', 'é', '\u{2027}', '\u{202a}'] {
            assert!(!is_control_or_separator(c), "{c:?}");
        }
    }

    #[test]
    fn an_escape_sequence_is_written_out_and_the_rest_kept() {
        assert_eq!(
            escape_controls("\u{1b}[2K\u{1b}[1Gok    fake\u{1b}[0m"),
            "\\u001B[2K\\u001B[1Gok    fake\\u001B[0m"
        );
        assert_eq!(escape_controls("a\u{2028}b\u{2029}"), "a\\u2028b\\u2029");
        assert_eq!(escape_controls("llama-server é"), "llama-server é");
    }

    #[test]
    fn a_format_character_that_reorders_a_line_is_written_out() {
        assert_eq!(
            escape_controls("12 MiB \u{202e}gpu-burn\u{2066}x\u{feff}"),
            "12 MiB \\u202Egpu-burn\\u2066x\\uFEFF"
        );
        for c in [
            '\u{61c}', '\u{200b}', '\u{200f}', '\u{202a}', '\u{202e}', '\u{2060}', '\u{2064}',
            '\u{2066}', '\u{2069}', '\u{feff}',
        ] {
            assert!(is_invisible_format(c), "{c:?}");
        }
        for c in [
            '\u{61b}', '\u{200a}', '\u{2010}', '\u{2029}', '\u{202f}', '\u{2065}', '\u{206a}',
            '\u{fefe}',
        ] {
            assert!(!is_invisible_format(c), "{c:?}");
        }
    }
}
