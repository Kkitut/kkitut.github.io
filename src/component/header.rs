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
                    // one level up, bottoming out at home (never above /)
                    let raw = window().location().pathname().unwrap_or_default();
                    let path = raw.trim_end_matches('/');
                    if path.is_empty() {
                        return;
                    }
                    let up = match path.rfind('/') {
                        Some(0) | None => "/",
                        Some(i) => &path[..i],
                    };
                    navigate(up, Default::default());
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
