#[cfg(test)]
mod tests {
    use crate::parser::{extract_tags, extract_task_links};

    #[test]
    fn extracts_simple_tags() {
        let result = extract_tags("今天 #工作 进展顺利 #灵感");
        assert_eq!(result.names, vec!["工作", "灵感"]);
    }

    #[test]
    fn lowercases_tags() {
        let result = extract_tags("#Work #WORK");
        assert_eq!(result.names, vec!["work", "work"]);
    }

    #[test]
    fn ignores_pure_numeric_tag() {
        let result = extract_tags("#123 和 #项目2");
        assert_eq!(result.names, vec!["项目2"]);
    }

    #[test]
    fn ignores_tag_with_punctuation() {
        let result = extract_tags("今天#工作，还有#灵感。");
        assert_eq!(result.names, vec!["工作", "灵感"]);
    }

    #[test]
    fn limits_tag_length() {
        let long = "a".repeat(51);
        let result = extract_tags(&format!("#{} #ok", long));
        assert_eq!(result.names, vec!["ok"]);
    }

    #[test]
    fn preserves_duplicate_tags() {
        let result = extract_tags("#工作 #工作 #学习");
        assert_eq!(result.names, vec!["工作", "工作", "学习"]);
    }

    #[test]
    fn extracts_task_links() {
        let links = extract_task_links("参见 [[完成设计]] 和 [[需求评审]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].raw_text, "完成设计");
        assert_eq!(links[1].raw_text, "需求评审");
    }

    #[test]
    fn extracts_task_link_with_spaces() {
        let links = extract_task_links("[[完成 v0.2.0 设计文档]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].raw_text, "完成 v0.2.0 设计文档");
    }

    #[test]
    fn handles_nested_brackets_as_single_link() {
        let links = extract_task_links("[[外层 [[内层]]]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].raw_text, "外层 [[内层");
    }

    #[test]
    fn returns_empty_for_no_links() {
        let links = extract_task_links("普通文本没有链接");
        assert!(links.is_empty());
    }

    #[test]
    fn records_task_link_positions() {
        let links = extract_task_links("参见 [[完成设计]] 和 [[需求评审]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].position, 7);
        assert_eq!(links[1].position, 28);
    }
}
