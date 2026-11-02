use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::page::{BlueSky, Err};

use super::blog::BLOG_POSTS;

/// Custom pages by name: /blogcustom/:name (plus anything under it).
/// A slug without a custom page 404s here; its home is /blog/:slug.
#[component]
pub fn BlogCustom() -> impl IntoView {
    let params = use_params_map();
    let post = Memo::new(move |_| {
        let slug = params.read().get("name").unwrap_or_default();
        BLOG_POSTS.iter().find(|post| post.slug == slug).copied()
    });

    view! {
        {move || match post.get() {
            Some(post) => match post.custom {
                Some("bluesky") => view! { <BlueSky/> }.into_any(),
                _ => view! { <Err code=404/> }.into_any(),
            },
            None => view! { <Err code=404/> }.into_any(),
        }}
    }
}
