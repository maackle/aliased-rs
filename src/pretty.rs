pub fn pretty_replace(text: &str, old: &str, replacement: &str) -> String {
    let pattern = pretty_pattern(old);
    regex::Regex::new(&pattern)
        .unwrap()
        .replace_all(&text, PrettyReplacer(&replacement))
        .to_string()
}

pub fn pretty_pattern(pretty_dbg: &str) -> String {
    pretty_dbg
        .split('\n')
        .map(|line| format!(" *{}", regex::escape(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub struct PrettyReplacer<'a>(pub &'a str);

impl<'a> regex::Replacer for PrettyReplacer<'a> {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        for cap in caps.iter() {
            let cap = cap.unwrap();
            let spaces = cap
                .as_str()
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            // let spaces = " ".repeat(cap.start() - 1);
            let r = self
                .0
                .split('\n')
                .map(|line| format!("{spaces}{line}"))
                .collect::<Vec<_>>()
                .join("\n");
            dst.push_str(&r);
        }
    }
}
