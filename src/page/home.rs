use leptos::prelude::*;

use crate::component::{Footer, Header};

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/home.css"/>

            <div id="app">
                <Header/>

                <main class="home-main">
                    <img
                        src="/res/image/Kkitut_NL.svg"
                        alt="Kkitut"
                        class="home-logo"
                    />
                </main>

                <Footer/>
            </div>
        </>
    }
}