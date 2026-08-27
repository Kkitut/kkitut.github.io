use leptos::{prelude::*, ev};

use crate::component::{Footer, Header};
use wasm_bindgen::{closure::Closure, JsCast};

#[derive(Clone, Copy, PartialEq)]
pub struct Section {
    pub id: &'static str,
    pub title: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Post {
    pub slug: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub date: &'static str,
    pub tag: &'static str,
    pub content: &'static str,
    pub sections: &'static [Section],
}

include!(concat!(env!("OUT_DIR"), "/blog.rs"));

#[component]
pub fn Blog() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (selected_tag, set_selected_tag) = signal(None::<&'static str>);

    let mut tags = BLOG_POSTS
        .iter()
        .map(|post| post.tag)
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    tags.sort_unstable();
    tags.dedup();

    let filtered_posts = Memo::new(move |_| {
        let search = search.get().trim().to_lowercase();
        let selected_tag = selected_tag.get();

        BLOG_POSTS
            .iter()
            .copied()
            .filter(|post| {
                let matches_search = search.is_empty()
                    || post.title.to_lowercase().contains(&search)
                    || post.desc.to_lowercase().contains(&search);

                let matches_tag = selected_tag
                    .map(|tag| post.tag == tag)
                    .unwrap_or(true);

                matches_search && matches_tag
            })
            .collect::<Vec<_>>()
    });

    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/blog.css"/>

            <div id="app">
                <Header/>

                <main class="blog-main">
                    <div class="blog-content">
                        <div class="blog-toolbar">
                            <input
                                class="blog-search"
                                type="text"
                                placeholder="Search"
                                prop:value=move || search.get()
                                on:input=move |ev| {
                                    set_search.set(event_target_value(&ev));
                                }
                            />

                            <div class="blog-tags">
                                <button
                                    class:active=move || selected_tag.get().is_none()
                                    on:click=move |_| {
                                        set_selected_tag.set(None);
                                    }
                                >
                                    "ALL"
                                </button>

                                {tags.into_iter().map(|tag| {
                                    view! {
                                        <button
                                            class:active=move || {
                                                selected_tag.get() == Some(tag)
                                            }
                                            on:click=move |_| {
                                                set_selected_tag.set(Some(tag));
                                            }
                                        >
                                            {format!("#{tag}")}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        <BlogList posts=filtered_posts/>
                    </div>
                </main>

                <Footer/>
            </div>
        </>
    }
}

#[component]
fn BlogList(posts: Memo<Vec<Post>>) -> impl IntoView {
    let grouped = Memo::new(move |_| {
        let posts = posts.get();

        let mut months = Vec::<(&str, Vec<Post>)>::new();

        for post in posts {
            let month = post.date.get(..7).unwrap_or(post.date);

            if let Some((_, month_posts)) =
                months.iter_mut().find(|(key, _)| *key == month)
            {
                month_posts.push(post);
            } else {
                months.push((month, vec![post]));
            }
        }

        months
    });

    view! {
        <div class="blog-list">
            <For
                each=move || grouped.get()
                key=|(month, _)| (*month).to_owned()
                children=move |(month, posts)| {
                    view! {
                        <section class="blog-month">
                            <h2 class="blog-month-title">
                                {format_month(month)}
                            </h2>

                            <div class="blog-month-posts">
                                {posts.into_iter().map(|post| {
                                    view! {
                                        <BlogCard post/>
                                    }
                                }).collect_view()}
                            </div>
                        </section>
                    }
                }
            />
        </div>
    }
}

#[component]
fn BlogCard(post: Post) -> impl IntoView {
    let href = format!("/blog/{}", post.slug);

    view! {
        <a class="blog-post" href=href>
            <div class="blog-post-top">
                <span class="blog-post-date">
                    {format_date(post.date)}
                </span>

                {(!post.tag.is_empty()).then(|| view! {
                    <span class="blog-post-tag">
                        {format!("#{}", post.tag)}
                    </span>
                })}
            </div>

            <div class="blog-post-content">
                <h2 class="blog-post-title">
                    {post.title}
                </h2>

                <p class="blog-post-desc">
                    {post.desc}
                </p>
            </div>

            <span class="blog-post-arrow">
                "→"
            </span>
        </a>
    }
}

#[component]
pub fn BlogPost(slug: String) -> impl IntoView {
    let post = BLOG_POSTS
        .iter()
        .find(|post| post.slug == slug);

    match post {
        Some(post) => {
            let sections = post.sections;

            let (progress, set_progress) = signal(0.0f64);

            let update_progress = move || {
                let window = window();
                let document = document();

                let scroll_top = window.scroll_y().unwrap_or(0.0);

                let viewport_height = window
                    .inner_height()
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(0.0);

                let document_height = document
                    .document_element()
                    .map(|element| element.scroll_height() as f64)
                    .unwrap_or(0.0);

                let max_scroll = (document_height - viewport_height).max(1.0);

                set_progress.set(
                    (scroll_top / max_scroll).clamp(0.0, 1.0)
                );
            };

            Effect::new(move |_| {
                update_progress();
            });

            window_event_listener(ev::scroll, move |_| {
                update_progress();
            });

            view! {
                <>
                    <link rel="stylesheet" href="/style/common.css"/>
                    <link rel="stylesheet" href="/style/header.css"/>
                    <link rel="stylesheet" href="/style/footer.css"/>
                    <link rel="stylesheet" href="/style/page/blog.css"/>
                    <link rel="stylesheet" href="/style/markdown.css"/>

                    <div id="app">
                        <Header/>

                        <main class="blog-main blog-detail-main">
                            <article class="blog-detail">
                                <div class="blog-detail-header">
                                    <div class="blog-detail-date">
                                        {format_date(post.date)}
                                    </div>

                                    <div class="blog-detail-title">
                                        {post.title}
                                    </div>
                                </div>

                                <article
                                    class="markdown blog-article"
                                    inner_html=post.content
                                />
                            </article>

                            {(!sections.is_empty()).then(|| view! {
                                <nav class="blog-section-nav">
                                    <div class="blog-section-nav-list">
                                        <div
                                            class="blog-section-nav-progress"
                                            style:height=move || {
                                                format!("{}%", progress.get() * 100.0)
                                            }
                                        />

                                        {sections.iter().map(|section| {
                                            view! {
                                                <a
                                                    href=format!("#{}", section.id)
                                                    on:click=move |ev| {
                                                        ev.prevent_default();

                                                        let Some(element) = document().get_element_by_id(section.id) else {
                                                            return;
                                                        };

                                                        let window = window();
                                                        let animation_window = window.clone();

                                                        let start = window.scroll_y().unwrap_or(0.0);
                                                        let target = start
                                                            + element.get_bounding_client_rect().top()
                                                            - 80.0;

                                                        let distance = target - start;
                                                        let duration = 150.0;

                                                        let start_time = std::rc::Rc::new(
                                                            std::cell::Cell::new(None::<f64>)
                                                        );

                                                        let animation = std::rc::Rc::new(
                                                            std::cell::RefCell::new(
                                                                None::<Closure<dyn FnMut(f64)>>
                                                            )
                                                        );

                                                        let animation_clone = animation.clone();
                                                        let start_time_clone = start_time.clone();

                                                        *animation.borrow_mut() = Some(Closure::new(
                                                            move |timestamp: f64| {
                                                                let initial = start_time_clone.get().unwrap_or_else(|| {
                                                                    start_time_clone.set(Some(timestamp));
                                                                    timestamp
                                                                });

                                                                let t = ((timestamp - initial) / duration)
                                                                    .clamp(0.0, 1.0);

                                                                let eased = (1.0 - (t - 1.0).powi(2)).sqrt();

                                                                animation_window.scroll_to_with_x_and_y(
                                                                    0.0,
                                                                    start + distance * eased,
                                                                );

                                                                if t < 1.0 {
                                                                    if let Some(callback) = animation_clone.borrow().as_ref() {
                                                                        let _ = animation_window.request_animation_frame(
                                                                            callback.as_ref().unchecked_ref(),
                                                                        );
                                                                    }
                                                                } else {
                                                                    animation_clone.borrow_mut().take();
                                                                }
                                                            },
                                                        ));

                                                        if let Some(callback) = animation.borrow().as_ref() {
                                                            let _ = window.request_animation_frame(
                                                                callback.as_ref().unchecked_ref(),
                                                            );
                                                        }
                                                    }
                                                >
                                                    {section.title}
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>
                                </nav>
                            })}
                        </main>

                        <Footer/>
                    </div>
                </>
            }.into_any()
        }

        None => view! {
            <crate::page::Err code=404/>
        }.into_any(),
    }
}

fn format_month(date: &str) -> String {
    let Some(month) = date.get(5..7) else {
        return date.to_owned();
    };

    let month = match month {
        "01" => "JAN",
        "02" => "FEB",
        "03" => "MAR",
        "04" => "APR",
        "05" => "MAY",
        "06" => "JUN",
        "07" => "JUL",
        "08" => "AUG",
        "09" => "SEP",
        "10" => "OCT",
        "11" => "NOV",
        "12" => "DEC",
        _ => return date.to_owned(),
    };

    format!("{} {}", &date[..4], month)
}

fn format_date(date: &str) -> String {
    let Some((year, month, day)) = date.split_once('-')
        .and_then(|(year, rest)| {
            rest.split_once('-').map(|(month, day)| (year, month, day))
        })
    else {
        return date.to_owned();
    };

    format!("{month}/{day}/{}", &year[2..])
}