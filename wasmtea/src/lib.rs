#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

use pulldown_cmark::{html, Parser};

#[cfg_attr(feature = "web", wasm_bindgen)]
pub fn render_markdown(markdown: String) -> String {
    let mut html_buf = String::new();
    let parser = Parser::new(&markdown[..]);
    html::push_html(&mut html_buf, parser);
    html_buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(render_markdown(String::new()), "");
    }

    #[test]
    fn test_heading() {
        let output = render_markdown("# Hello".to_string());
        assert_eq!(output, "<h1>Hello</h1>\n");
    }

    #[test]
    fn test_paragraph() {
        let output = render_markdown("Hello world".to_string());
        assert_eq!(output, "<p>Hello world</p>\n");
    }

    #[test]
    fn test_bold() {
        let output = render_markdown("**bold**".to_string());
        assert_eq!(output, "<p><strong>bold</strong></p>\n");
    }

    #[test]
    fn test_italic() {
        let output = render_markdown("*italic*".to_string());
        assert_eq!(output, "<p><em>italic</em></p>\n");
    }

    #[test]
    fn test_inline_code() {
        let output = render_markdown("`code`".to_string());
        assert_eq!(output, "<p><code>code</code></p>\n");
    }

    #[test]
    fn test_link() {
        let output = render_markdown("[text](https://example.com)".to_string());
        assert_eq!(output, "<p><a href=\"https://example.com\">text</a></p>\n");
    }

    #[test]
    fn test_unordered_list() {
        let output = render_markdown("- one\n- two\n- three".to_string());
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>one</li>"));
        assert!(output.contains("<li>two</li>"));
        assert!(output.contains("<li>three</li>"));
    }

    #[test]
    fn test_ordered_list() {
        let output = render_markdown("1. first\n2. second".to_string());
        assert!(output.contains("<ol>"));
        assert!(output.contains("<li>first</li>"));
        assert!(output.contains("<li>second</li>"));
    }

    #[test]
    fn test_fenced_code_block() {
        let output = render_markdown("```\nlet x = 1;\n```".to_string());
        assert!(output.contains("<pre><code>"));
        assert!(output.contains("let x = 1;"));
    }

    #[test]
    fn test_blockquote() {
        let output = render_markdown("> quoted text".to_string());
        assert!(output.contains("<blockquote>"));
        assert!(output.contains("quoted text"));
    }

    #[test]
    fn test_horizontal_rule() {
        let output = render_markdown("---".to_string());
        assert!(output.contains("<hr"));
    }

    #[test]
    fn test_multiple_headings() {
        let md = "# H1\n## H2\n### H3".to_string();
        let output = render_markdown(md);
        assert!(output.contains("<h1>H1</h1>"));
        assert!(output.contains("<h2>H2</h2>"));
        assert!(output.contains("<h3>H3</h3>"));
    }
}
