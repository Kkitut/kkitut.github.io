use leptos::{ev, prelude::*};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

mod component;
mod page;

// How many client-side pushes since the last "/" (saturating).
// Home bar goes history.back() only while this is above zero,
// so in-app back never leaves the site.
thread_local! {
    static DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LAST_PATH: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
    static POPPED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FLYING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn back_depth() -> usize {
    DEPTH.with(|depth| depth.get())
}

pub(crate) fn try_fly() -> bool {
    FLYING.with(|flying| !flying.replace(true))
}

#[component]
fn ScrollTop() -> impl IntoView {
    let loc = use_location();
    // MPA parity: start at the top on every navigation
    Effect::new(move |_| {
        let path = loc.pathname.get();
        FLYING.with(|flying| flying.set(false));
        let prev = LAST_PATH.with(|last| last.borrow_mut().replace(path.clone()));
        if prev.is_some() {
            if POPPED.with(|popped| popped.replace(false)) {
                DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
            } else if path == "/" {
                DEPTH.with(|depth| depth.set(0));
            } else {
                DEPTH.with(|depth| depth.set(depth.get() + 1));
            }
        }
        if let Some(el) = document().document_element() {
            el.set_scroll_top(0);
        }
    });
    window_event_listener(ev::popstate, move |_| {
        POPPED.with(|popped| popped.set(true));
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
                <Route path=path!("/custom/:name") view=page::BlogCustom/>
                <Route path=path!("/custom/:name/*rest") view=page::BlogCustom/>
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
