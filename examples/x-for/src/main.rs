use hirola::{prelude::*, signal_vec::{MutableVec, SignalVecExt}};
use web_sys::Event;

fn colors() -> DomBuilder {
    let colors = MutableVec::new_with_values(vec!["Red", "Green", "Blue", "Violet"]);
    let add_new = colors.update_with(move |colors, _e: Event| {
        colors.lock_mut().push("Violet-Dark");
    });

    html! {
        <>
            <h2>"Static"</h2>
            <ul>
                {for (_index, item) in (0..3).enumerate() {
                    html! { <li>{item.to_string()}</li> }
                }}
            </ul>
            <h2>"Reactive"</h2>
            <ul>
                {colors
                    .signal_vec()
                    .render_map(|item| {
                        html! { <li>{item}</li> }
                    })}
            </ul>
            <h2>"Reactive Filtered Starts with V"</h2>
            <ul>
                {colors
                    .signal_vec()
                    .filter(|color| color.starts_with("V"))
                    .render_map(|item| {
                        html! { <li>{item}</li> }
                    })}
            </ul>
            <button on:click=add_new>"Add New Color"</button>
        </>
    }
}

fn main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let view = render_to(colors(), &body).unwrap();

    std::mem::forget(view);
}
