use leptos::{ev, prelude::*};
use wasm_bindgen::JsCast;

#[component]
pub fn Day(qid: &'static str) -> impl IntoView {
    // scroll reveals: mark [data-reveal] visible once it enters the viewport
    let reveal = move || {
        let vh = window()
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0);
        let Ok(list) = document().query_selector_all("[data-reveal]") else {
            return;
        };
        for index in 0..list.length() {
            let Some(node) = list.item(index) else {
                continue;
            };
            let element: leptos::web_sys::HtmlElement = node.unchecked_into();
            if element.get_bounding_client_rect().top() < vh * 0.82 {
                let _ = element.class_list().add_1("in");
            }
        }
    };

    Effect::new(move |_| {
        reveal();
    });

    window_event_listener(ev::scroll, move |_| {
        reveal();
    });

    let staring = "멍해질 만큼 넓은 우주,";

    view! {
        <>
            <link rel="stylesheet" href="/style/page/day/main.css"/>

            <p class="day-kicker" data-reveal>"대기권"</p>

            <p class="day-lead" data-reveal>"지구의 탄생 이래 이어진 파란 하늘."</p>
            <p class="day-body" data-reveal>"하늘이 파란 건 지구의 탄생부터 정해진 물리 법칙이다."</p>
            <p class="day-body" data-reveal style="transition-delay:.1s">
                "빛이 대기에 부딪혀 산란하는 자연 현상."
            </p>
            <p class="day-body" data-reveal style="transition-delay:.2s">
                "비가 오고 눈이 내리는 것 역시 대단할 것 없는 기상 작용일 뿐이다."
            </p>

            <div class="day-strip" data-reveal aria-hidden="true">
                <div class="day-strip-inner">
                    <span>"빛이 대기에 부딪혀 산란하는 자연 현상."</span>
                    <span>"빛이 대기에 부딪혀 산란하는 자연 현상."</span>
                    <span>"빛이 대기에 부딪혀 산란하는 자연 현상."</span>
                </div>
            </div>

            <section class="day-act day-white">
                <p class="day-lead" data-reveal>"그런데도 가끔 걸음을 멈춘다."</p>
                <div class="day-horizon" data-reveal aria-hidden="true"/>
                <p class="day-body" data-reveal>"지평선 위로 펼쳐진 무심한 그라데이션이 현실감을 앗아간다."</p>
                <p class="day-body" data-reveal style="transition-delay:.1s">
                    "푸른 장막 아래서 숨을 들이켜면,"
                </p>
                <p class="day-body" data-reveal style="transition-delay:.2s">
                    "내가 살아있다는 사실과,"
                </p>
                <p class="day-body" data-reveal style="transition-delay:.3s">
                    "이 지루한 사회가 여전히 굴러가고 있다는 사실이 끈적하게 체감된다."
                </p>
            </section>

            <section class="day-act day-black">
                <div class="day-ring" data-reveal aria-hidden="true"/>
                <h2 class="day-question" id=qid data-reveal>
                    "우리가 '특별하다'고 명명한 것들은 정말로 특별한 것인가?"
                </h2>
                <p class="day-body" data-reveal>
                    "우주적 관점에서 보면 금이나 은 따위는 발에 채는 돌덩이보다 흔한 물질이다."
                </p>
                <p class="day-body" data-reveal style="transition-delay:.1s">
                    "단지 이 행성에서 구하기 피곤할 뿐이다."
                </p>
                <p class="day-body" data-reveal style="transition-delay:.2s">"반면 맑은 땅,"</p>
                <p class="day-body" data-reveal style="transition-delay:.3s">
                    "마실 수 있는 공기, 파란 하늘은 전 우주를 뒤져도 찾기 힘든 소중한 희귀품이다."
                </p>
                <p class="day-body" data-reveal style="transition-delay:.4s">
                    "공허 속에서 이것들은 사치에 가깝다."
                </p>
            </section>

            <section class="day-act day-gray">
                <p class="day-lead" data-reveal>"글을 읽어보는 독자, 그래 너."</p>
                <p class="day-body" data-reveal style="transition-delay:.1s">
                    "지금 이걸 보고있는 너. 혹은 당신으로 칭해야 할까."
                </p>
                <p class="day-spell" data-reveal aria-label="멍해질 만큼 넓은 우주,">
                    {staring
                        .chars()
                        .enumerate()
                        .map(|(index, ch)| {
                            let ch = if ch == ' ' {
                                "\u{a0}".to_string()
                            } else {
                                ch.to_string()
                            };
                            view! {
                                <span
                                    class="day-ch"
                                    style=format!(
                                        "animation-delay:{:.2}s",
                                        index as f32 * 0.05
                                    )
                                >
                                    {ch}
                                </span>
                            }
                        })
                        .collect_view()}
                </p>
                <p class="day-body" data-reveal>"그 속의 작은 점 하나인 지구,"</p>
                <p class="day-body" data-reveal style="transition-delay:.1s">
                    "그 안에서도 더 작은 사이트에 머무는 존재."
                </p>
                <p class="day-body" data-reveal style="transition-delay:.2s">"이성적으로 따져보자."</p>
                <p class="day-body" data-reveal style="transition-delay:.3s">
                    "당신이 지금 이 순간 존재할 확률이 얼마나 되겠나.."
                </p>
                <p class="day-lead" data-reveal>"묻고 싶다."</p>
                <p class="day-body" data-reveal style="transition-delay:.1s">
                    "당신은 셀수 없는 확률을 어떻게 소모하고 있나."
                </p>
            </section>

            <section class="day-act day-white">
                <p class="day-body" data-reveal>
                    "남들이 거들떠보지도 않는 평범한 일상에 단 한 번이라도 눈길을 준 적이 있는가."
                </p>
                <p class="day-body" data-reveal style="transition-delay:.15s">
                    "모두가 특별함을 좇을 때,"
                </p>
                <p class="day-body" data-reveal style="transition-delay:.3s">
                    "아무것도 아닌 것에 재미를 느끼며 사는 것도 나쁘지 않지 않은가?"
                </p>
                <p class="day-lead" data-reveal>
                    "어쩌면 그게 가장 행복한 생존 방식일지도 모른다고 생각한다."
                </p>
            </section>

            <section class="day-act day-black day-finale">
                <p class="day-body" data-reveal>"정말 귀한 건,"</p>
                <p class="day-body" data-reveal style="transition-delay:.15s">
                    "우리가 매일 생각 없이 마주하는 저 파란 하늘이니까."
                </p>
                <p class="day-final-title" data-reveal>"파란 하늘"</p>
                <p class="day-title-en" data-reveal style="transition-delay:.2s">
                    "Blue Sky"
                </p>
            </section>
        </>
    }
}
