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
