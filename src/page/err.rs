use leptos::prelude::*;

use crate::component::{Footer, Header};

#[component]
pub fn Err(code: u16) -> impl IntoView {
    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/err.css"/>

            <div id="app">
                <Header/>

                <main class="err-main">
                    <div class="err-content">
                        <span>"!"</span>
                        <small>{code}</small>
                    </div>
                </main>

                <Footer/>
            </div>
        </>
    }
}