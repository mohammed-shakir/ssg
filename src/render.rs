use crate::content::Document;
use pulldown_cmark::{Options, Parser, html};

pub fn render_html<M>(doc: &Document<M>) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&doc.body, options);

    let mut html_output = String::with_capacity(doc.body.len() * 3 / 2);
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn render_html_sanitized<M>(doc: &Document<M>) -> String {
    let html = render_html(doc);
    ammonia::clean(&html).to_string()
}
