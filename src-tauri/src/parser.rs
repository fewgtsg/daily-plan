use regex::Regex;
use std::sync::OnceLock;

pub struct ParsedTags {
    pub names: Vec<String>,
}

pub struct ParsedTaskLink {
    pub raw_text: String,
    pub position: usize,
}

fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#([\p{L}\p{N}_-]+)").unwrap())
}

fn task_link_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Use non-greedy match so nested brackets are not parsed as multiple links
    RE.get_or_init(|| Regex::new(r"\[\[(.+?)\]\]").unwrap())
}

pub fn extract_tags(text: &str) -> ParsedTags {
    let mut names = Vec::new();
    for cap in tag_regex().captures_iter(text) {
        let name = cap[1].to_lowercase();
        if !name.chars().all(|c| c.is_ascii_digit()) && name.chars().count() <= 50 {
            names.push(name);
        }
    }
    ParsedTags { names }
}

pub fn extract_task_links(text: &str) -> Vec<ParsedTaskLink> {
    task_link_regex()
        .captures_iter(text)
        .map(|cap| {
            let m = cap.get(0).unwrap();
            ParsedTaskLink {
                raw_text: cap[1].trim().to_string(),
                position: m.start(),
            }
        })
        .collect()
}
