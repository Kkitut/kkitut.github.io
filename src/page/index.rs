use leptos::prelude::*;

use crate::component::{Footer, Header};

#[component]
pub fn Index() -> impl IntoView {
    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/index.css"/>

            <div id="app">
                <Header/>

                <main class="index-main">
                    <div class="index-wrap">
                        <h1 class="index-hero">"Index"</h1>
                        <p class="index-specimen">"둘러보기"</p>

                        <nav class="index-blocks">
                            <a class="index-block" href="/blog">
                                <div class="index-block-main">
                                    <span class="index-block-no">"01"</span>
                                    <span class="index-block-name">"Blog"</span>
                                    <span class="index-block-meta">"/blog"</span>
                                </div>
                                <p class="index-block-side">
                                    "책은 거들떠보지도 않지만,"
                                    <br/>
                                    "글은 남기고 싶었습니다"
                                </p>
                                <svg class="index-ants index-ants-cw" aria-hidden="true"><rect x="1" y="1" width="100%" height="100%" rx="7"/></svg>
                            </a>

                            <a class="index-block" href="/bot">
                                <div class="index-block-main">
                                    <span class="index-block-no">"02"</span>
                                    <span class="index-block-name">"Bot"</span>
                                    <span class="index-block-meta">"/bot"</span>
                                </div>
                                <p class="index-block-side">
                                    "몇줄 코드로 무언가를 만든다는거."
                                    <br/>
                                    "생각해보면 신기하지 않아요?"
                                </p>
                                <svg class="index-ants index-ants-ccw" aria-hidden="true"><rect x="1" y="1" width="100%" height="100%" rx="7"/></svg>
                            </a>
                        </nav>

                        <div class="index-strip">
                            "× × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × × ×"
                        </div>
                    </div>
                </main>

                <Footer/>
            </div>
        </>
    }
}
