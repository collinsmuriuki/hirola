mod model;
use hirola::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

fn fetch_users(app: &HirolaApp) -> TemplateResult<DomNode> {
    let msg = Signal::new(String::from("Loading"));
    let message = msg.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let url = format!("https://jsonplaceholder.typicode.com/users");

        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        let text = resp.text().unwrap();
        let text = JsFuture::from(text).await.unwrap();

        message.set(text.as_string().unwrap());
    });

    html! {
            <div class="grid h-screen place-items-center">
                <div class="h-10 w-32">
                    {msg.get()}
                </div>
           </div>
    }
}

fn main() {
    let app = HirolaApp::new();
    app.mount("body", fetch_users);
}