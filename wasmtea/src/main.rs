extern crate structopt;

use structopt::StructOpt;
use std::path::PathBuf;
use std::fs;

#[derive(StructOpt)]
#[structopt(name = "rust_wasi_markdown_parser", about = "Markdown to HTML renderer CLI, written with Rust & WASI")]
pub struct Options {
    /// The markdown file to render
    #[structopt(parse(from_os_str))]
    filename: PathBuf,
}

fn main() {
    let options = Options::from_args();
    let contents = fs::read_to_string(options.filename)
        .expect("Something went wrong reading the file");
    let result = wasmtea::render_markdown(contents);
    println!("{}", result);
}
