use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(source: &str) -> String {
    let mut options = Options::empty();

    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options);

    let mut output = String::new();
    html::push_html(&mut output, parser);

    output
}

#[component]
pub fn Markdown(source: &'static str) -> impl IntoView {
    let html = render_markdown(source);

    view! {
        <div class="markdown" inner_html=html/>
    }
}