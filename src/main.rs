use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

mod component;
mod page;

// Previous in-app paths, oldest last. Home bar pops one per tap and
// bottoms out at home: back navigation never leaves the site.
thread_local! {
    static LAST_PATH: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
    static BACK_STACK: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn pop_path() -> Option<String> {
    BACK_STACK.with(|stack| stack.borrow_mut().pop())
}

#[component]
fn ScrollTop() -> impl IntoView {
    let loc = use_location();
    Effect::new(move |_| {
        let path = loc.pathname.get();
        if let Some(prev) = LAST_PATH.with(|last| last.borrow_mut().replace(path.clone())) {
            if prev != path {
                BACK_STACK.with(|stack| stack.borrow_mut().push(prev));
            }
        }
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
