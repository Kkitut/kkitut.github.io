use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header>
            <a class="menu-hitbox" href="/index">
                <span class="menu-text">"Index"</span>
            </a>

            <a class="home-hitbox" href="/">
                <div class="home-bar"/>
            </a>

            <a class="sns-hitbox" href="/sns">
                <span class="sns-text">"SNS"</span>
            </a>
        </header>
    }
}
