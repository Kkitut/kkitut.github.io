use leptos::prelude::*;

use crate::component::{Footer, Header};

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[component]
pub fn Bot() -> impl IntoView {
    // /bot fits in the viewport: lock scrolling, restore on leave
    Effect::new(move |_| {
        if let Some(body) = document().body() {
            let _ = body.style().set_property("overflow", "hidden");
        }
    });
    on_cleanup(move || {
        if let Some(body) = document().body() {
            let _ = body.style().set_property("overflow", "");
        }
    });

    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/bot.css"/>

            <div id="app">
                <Header/>

                <main class="bot-main">
                    <div class="bot-wrap">
                        <h1 class="bot-title">"Bots"</h1>
                        <p class="bot-sub">"OpenSource"</p>

                        <div class="bot-grid">
                            <a class="bot-card bot-wide bot-rustoki" href="/bot/rustoki">
                                <div>
                                    <h2>"Rustoki"</h2>
                                    <p>"잡다한 도구들 모음.."</p>
                                </div>
                                <span>"KkitutLab/Rustoki"</span>
                            </a>

                            <a
                                class="bot-card bot-wide"
                                href="https://github.com/KkitutLab/Eara"
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <div>
                                    <h2>"Eara"</h2>
                                    <p>"들낙 잡으려고 개발한 것"</p>
                                </div>
                                <span>"KkitutLab/Eara"</span>
                            </a>
                        </div>
                    </div>
                </main>

                <Footer/>
            </div>
        </>
    }
}

#[component]
pub fn Rustoki() -> impl IntoView {
    let delay = format!("animation-delay:-{:.3}s", random() * 92.3);

    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/bot.css"/>

            <div id="app">
                <Header/>

                <main class="rk-page">
                    <div class="rk-hero-wrap">
                        <p class="rk-kicker">"KkitutLab"</p>
                        <h1 class="rk-hero">"Rustoki"</h1>
                        <p class="rk-mark">"<\""</p>
                        <p class="rk-lead">"Rust를 써보기 위해"</p>
                    </div>

                    <div class="rk-strip"></div>

                        <section id="intro" class="rk-sec">
                            <p class="rk-q" style=delay>"?"</p>
                            <h2 class="rk-sec-title rk-align-right">"어쩌다가.."</h2>
                            <p class="rk-body rk-align-right">
                                "누가 러스트를 써보라고 추천했었는데,"
                                <br/>
                                "보통 튜토리얼은 질색이라서"
                            </p>
                            <p class="rk-body rk-align-right">
                                "그러다가 디스코드 봇이라는 세계에 접하게 되었습니다."
                            </p>
                        </section>

                        <section id="misc" class="rk-sec">
                            <p class="rk-bang">"!"</p>
                            <h2 class="rk-sec-title">"잡다한 애"</h2>
                            <p class="rk-body">
                                "별의별것을 막 넣었었는데"
                                <br/>
                                "임베드 만들 수 있는 명령어,"
                                <br/>
                                "생일축하, DM연결, 메시지 json보기.."
                                <br/>
                                "리팩토링 이전은 상상도 하기 싫네요"
                            </p>
                        </section>

                        <section id="voice" class="rk-sec">
                            <div class="rk-rule"></div>

                            <div class="rk-drum">
                                <div class="rk-drum-inner">
                                    <span>"Rustoki"</span>
                                    <span>"러스토키"</span>
                                    <span>"<\""</span>
                                    <span aria-hidden="true">"Rustoki"</span>
                                </div>
                            </div>

                            <p class="rk-body rk-align-center">
                                "라고 부르면.."
                                <br/>
                                "DB에서 저장된 무언가를 불러와 대답합니다."
                                <br/>
                                "뭔가 챗봇같은걸 만들고 싶었는데 AI는 힘들었어요"
                            </p>
                        </section>

                        <section id="invite" class="rk-sec">
                            <div class="bot-links rk-links-center">
                                <a
                                    href="https://github.com/KkitutLab/Rustoki"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                >
                                    "KkitutLab/Rustoki →"
                                </a>
                            </div>

                            <p class="rk-note rk-align-center">"MIT"</p>
                        </section>
                </main>

                <Footer/>
            </div>
        </>
    }
}
