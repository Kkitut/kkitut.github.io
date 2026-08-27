use std::{env, fs, path::PathBuf};

use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

struct Section {
    id: String,
    title: String,
}

fn slugify(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn render_markdown(source: &str) -> (String, Vec<Section>) {
    let mut options = Options::empty();

    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options);

    let mut events = Vec::new();
    let mut sections = Vec::new();
    let mut heading = None::<String>;

    for event in parser {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                heading = Some(String::new());
                events.push(Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }));
            }

            Event::Text(text) if heading.is_some() => {
                if let Some(title) = heading.as_mut() {
                    title.push_str(&text);
                }

                events.push(Event::Text(text));
            }

            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                if let Some(title) = heading.take() {
                    let title = title.trim().to_owned();
                    let id = slugify(&title);

                    sections.push(Section {
                        id,
                        title,
                    });
                }

                events.push(Event::End(TagEnd::Heading(HeadingLevel::H2)));
            }

            event => {
                events.push(event);
            }
        }
    }

    let mut content = String::new();
    html::push_html(&mut content, events.into_iter());

    for section in &sections {
        let target = format!("<h2>{}</h2>", section.title);
        let replacement = format!(
            r#"<h2 id="{}">{}</h2>"#,
            section.id,
            section.title
        );

        content = content.replacen(&target, &replacement, 1);
    }

    (content, sections)
}

fn main() {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    let blog_dir = manifest_dir.join("content/blog");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut posts = Vec::new();

    for entry in fs::read_dir(&blog_dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();

        let Some(front) = source.strip_prefix("+++\n") else {
            continue;
        };

        let Some((front, body)) = front.split_once("\n+++\n") else {
            continue;
        };

        let mut title = String::new();
        let mut desc = String::new();
        let mut date = String::new();
        let mut tag = String::new();

        for line in front.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            let value = value.trim().trim_matches('"');

            match key.trim() {
                "title" => title = value.to_owned(),
                "desc" => desc = value.to_owned(),
                "date" => date = value.to_owned(),
                "tag" => tag = value.to_owned(),
                _ => {}
            }
        }

        let slug = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();

        let (content, sections) = render_markdown(body);

        posts.push((
            slug,
            title,
            desc,
            date,
            tag,
            content,
            sections,
        ));
    }

    posts.sort_by(|a, b| b.3.cmp(&a.3));

    let mut output = String::from(
        "pub static BLOG_POSTS: &[Post] = &[\n",
    );

    for (
        slug,
        title,
        desc,
        date,
        tag,
        content,
        sections,
    ) in posts {
        output.push_str(&format!(
            "    Post {{ slug: {:?}, title: {:?}, desc: {:?}, date: {:?}, tag: {:?}, content: {:?}, sections: &[",
            slug,
            title,
            desc,
            date,
            tag,
            content,
        ));

        for section in sections {
            output.push_str(&format!(
                "Section {{ id: {:?}, title: {:?} }},",
                section.id,
                section.title,
            ));
        }

        output.push_str("] },\n");
    }

    output.push_str("];\n");

    fs::write(
        out_dir.join("blog.rs"),
        output,
    )
    .unwrap();

    println!("cargo:rerun-if-changed=content/blog");
}