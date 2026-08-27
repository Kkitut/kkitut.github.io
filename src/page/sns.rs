use leptos::prelude::*;

use crate::component::{Footer, Header};

#[component]
pub fn Sns() -> impl IntoView {
    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/sns.css"/>

            <div id="app" class="sns-page">
                <Header/>

                <main class="sns-main">
                    <div class="sns-grid">
                        <div class="sns-tile sns-github">
                            <a
                                class="sns-github-main"
                                href="https://github.com/Kkitut"
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <div>
                                    <h2>"GitHub"</h2>
                                    <p>"더러운 코드저장소, GPL v3 선호"</p>
                                </div>

                                <span>"github.com/Kkitut"</span>

                                <img
                                    class="sns-logo"
                                    src="/res/image/github.svg"
                                    alt=""
                                />
                            </a>

                            <a
                                class="sns-github-org"
                                href="https://github.com/KkitutLab"
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <div>
                                    <h3>"KkitutLab"</h3>
                                    <p>"Organization"</p>
                                </div>

                                <span>"github.com/KkitutLab"</span>

                                <img
                                    class="sns-logo"
                                    src="/res/image/github.svg"
                                    alt=""
                                />
                            </a>
                        </div>

                        <a
                            class="sns-tile sns-youtube"
                            href="https://youtube.com/@Kkitut"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            <div>
                                <h2>"YouTube"</h2>
                                <p>"아무거나 업로드"</p>
                            </div>

                            <span>"youtube.com/@Kkitut"</span>

                            <img
                                class="sns-logo"
                                src="/res/image/youtube.svg"
                                alt=""
                            />
                        </a>

                        <a
                            class="sns-tile sns-discord"
                            href="https://discord.com/invite/SY4HD9pmzq"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            <div>
                                <h2>"Discord"</h2>
                                <p>"목적없는 서버, 디스코드 ID는 kkitut"</p>
                            </div>

                            <span>"discord.com/invite/SY4HD9pmzq"</span>

                            <img
                                class="sns-logo"
                                src="/res/image/discord.svg"
                                alt=""
                            />
                        </a>
                    </div>
                </main>

                <Footer/>
            </div>
        </>
    }
}