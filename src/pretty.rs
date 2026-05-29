pub(crate) fn pretty_regex(pretty_dbg: &str) -> regex::Regex {
    let pattern = pretty_dbg
        .split('\n')
        .map(|line| format!(" *{}", regex::escape(line)))
        .collect::<Vec<_>>()
        .join("\n");
    regex::Regex::new(&pattern).expect("pretty pattern is always a valid regex")
}

pub(crate) fn pretty_replace(regex: &regex::Regex, text: &str, replacement: &str) -> String {
    regex
        .replace_all(text, PrettyReplacer(replacement))
        .to_string()
}

struct PrettyReplacer<'a>(&'a str);

impl<'a> regex::Replacer for PrettyReplacer<'a> {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let matched = caps.get(0).expect("a match always has group 0").as_str();
        let spaces: String = matched.chars().take_while(|c| c.is_whitespace()).collect();
        let replaced = self
            .0
            .split('\n')
            .map(|line| format!("{spaces}{line}"))
            .collect::<Vec<_>>()
            .join("\n");
        dst.push_str(&replaced);
    }
}
