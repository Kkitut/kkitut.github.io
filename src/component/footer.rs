use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer>
            <a
                class="footer-link"
                href="/me"
            >
                "© Kkitut"
            </a>

            <a
                class="footer-link"
                href="https://creativecommons.org/licenses/by/4.0/"
                target="_blank"
                rel="noopener noreferrer"
            >
                "CC BY 4.0"
            </a>
        </footer>
    }
}