use leptos::prelude::*;

use crate::component::{Footer, Header, Markdown};

const CONTENT: &str = include_str!("../../content/me.md");

#[component]
pub fn Me() -> impl IntoView {
    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/markdown.css"/>

            <div id="app">
                <Header/>

                <main>
                    <Markdown source=CONTENT/>
                </main>

                <Footer/>
            </div>
        </>
    }
}