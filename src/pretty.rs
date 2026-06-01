//! Pretty (`{:#?}`) substitution.
//!
//! Pretty output interleaves indentation that varies with nesting depth, so a
//! registered value's debug string is not a fixed substring of the formatted
//! output — it is a sequence of lines each carrying arbitrary leading
//! whitespace. We therefore match with regexes rather than plain strings.
//!
//! All registered patterns are combined into a single alternation regex so the
//! whole output is scanned in one pass instead of once per registered alias.

/// Build the regex pattern for one pretty (`{:#?}`) debug string: each line is
/// escaped and allowed to carry arbitrary leading indentation.
pub(crate) fn pretty_pattern(pretty_dbg: &str) -> String {
    pretty_dbg
        .split('\n')
        .map(|line| format!(" *{}", regex::escape(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Combine many pretty patterns into a single alternation regex. Each pattern
/// is wrapped in a capture group so the matched alternative can be identified:
/// capture group `i + 1` corresponds to `patterns[i]`. Patterns should be
/// ordered longest-first so the longest candidate wins regardless of the
/// engine's alternation precedence.
pub(crate) fn combined_pretty_regex(patterns: &[String]) -> regex::Regex {
    let alternation = patterns
        .iter()
        .map(|p| format!("({p})"))
        .collect::<Vec<_>>()
        .join("|");
    regex::Regex::new(&alternation).expect("pretty patterns are always valid regexes")
}

/// Replace every pretty match in `text` in one pass, dispatching each match to
/// the replacement for the alternative that matched and re-indenting it to the
/// match's leading whitespace. `replacements` is parallel to the patterns the
/// regex was built from.
pub(crate) fn pretty_replace_all(
    regex: &regex::Regex,
    text: &str,
    replacements: &[String],
) -> String {
    regex
        .replace_all(text, PrettyReplacer(replacements))
        .to_string()
}

struct PrettyReplacer<'a>(&'a [String]);

impl<'a> regex::Replacer for PrettyReplacer<'a> {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let matched = caps.get(0).expect("a match always has group 0").as_str();
        // Exactly one alternative's capture group is set; its index selects the
        // replacement. (Patterns are built from `regex::escape`d lines joined
        // by ` *`, so they contain no nested groups to shift the indices.)
        let idx = (1..caps.len())
            .find(|&i| caps.get(i).is_some())
            .expect("exactly one alternative matches")
            - 1;
        let spaces: String = matched.chars().take_while(|c| c.is_whitespace()).collect();
        let replaced = self.0[idx]
            .split('\n')
            .map(|line| format!("{spaces}{line}"))
            .collect::<Vec<_>>()
            .join("\n");
        dst.push_str(&replaced);
    }
}
