use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header>
            <div
                class="menu-hitbox"
                on:click=move |_| {
                    let _ = web_sys::window()
                        .unwrap()
                        .location()
                        .set_href("/blog");
                }
            >
                <span class="menu-text">"Blog"</span>
            </div>

            <div
                class="home-hitbox"
                on:click=move |_| {
                    let _ = web_sys::window()
                        .unwrap()
                        .location()
                        .set_href("/");
                }
            >
                <div class="home-bar"/>
            </div>

            <div
                class="sns-hitbox"
                on:click=move |_| {
                    let _ = web_sys::window()
                        .unwrap()
                        .location()
                        .set_href("/sns");
                }
            >
                <span class="sns-text">"SNS"</span>
            </div>
        </header>
    }
}