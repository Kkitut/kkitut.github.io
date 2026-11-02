use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn Header() -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <header>
            <a class="menu-hitbox" href="/index">
                <span class="menu-text">"Index"</span>
            </a>

            <a
                class="home-hitbox"
                href="/"
                on:click=move |ev| {
                    ev.prevent_default();
                    // in-app back stack, bottoms out at home (never leaves the site)
                    match crate::pop_path() {
                        Some(path) => navigate(&path, Default::default()),
                        None => navigate("/", Default::default()),
                    }
                }
            >
                <div class="home-bar"/>
            </a>

            <a class="sns-hitbox" href="/sns">
                <span class="sns-text">"SNS"</span>
            </a>
        </header>
    }
}
