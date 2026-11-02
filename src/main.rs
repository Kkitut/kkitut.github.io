use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

mod component;
mod page;

#[component]
fn ScrollTop() -> impl IntoView {
    let loc = use_location();
    // MPA parity: start at the top on every navigation
    Effect::new(move |_| {
        loc.pathname.get();
        if let Some(el) = document().document_element() {
            el.set_scroll_top(0);
        }
    });
    view! { <></> }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <ScrollTop/>
            <Routes fallback=|| view! { <page::Err code=404/> }>
                <Route path=path!("/") view=page::Home/>
                <Route path=path!("/index") view=page::Index/>
                <Route path=path!("/blog") view=page::Blog/>
                <Route path=path!("/blog/:slug") view=page::BlogPost/>
                <Route path=path!("/bot") view=page::Bot/>
                <Route path=path!("/bot/rustoki") view=page::Rustoki/>
                <Route path=path!("/sns") view=page::Sns/>
                <Route path=path!("/me") view=page::Me/>
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(App)
}
