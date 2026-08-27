use leptos::prelude::*;
use leptos::web_sys;

mod component;
mod page;

fn main() {
    mount_to_body(|| {
        let path = web_sys::window()
            .unwrap()
            .location()
            .pathname()
            .unwrap_or_default();

        match path.as_str() {
           "/blog" => view! {
                <page::Blog/>
            }.into_any(),

            path if path.starts_with("/blog/") => {
                let slug = path.trim_start_matches("/blog/");

                view! {
                    <page::BlogPost slug=slug.to_owned()/>
                }.into_any()
            }

            "/sns" => view! {
                <page::Sns/>
            }.into_any(),

            "/me" => view! {
                <page::Me/>
            }.into_any(),

            "/" => view! {
                <page::Home/>
            }.into_any(),

            _ => view! {
                <page::Err code=404/>
            }.into_any(),
        }
    });
}