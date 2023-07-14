use crate::{builder::ViewBuilder, prelude::*, view::View};
use futures_signals::signal::{Mutable, MutableSignalCloned, SignalExt};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};
#[cfg(feature = "dom")]
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
#[cfg(feature = "dom")]
use web_sys::{Element, Event};

#[derive(Clone, Debug)]
pub struct RouterHandle {
    current: Mutable<String>,
}
impl RouterHandle {
    pub fn push(&self, path: &str) {
        #[cfg(feature = "dom")]
        let window = web_sys::window().unwrap();
        #[cfg(feature = "dom")]
        window
            .history()
            .unwrap()
            .push_state_with_url(&JsValue::default(), "", Some(&path))
            .unwrap();
        self.current.set(path.to_owned());
    }

    pub fn link(&self) -> Box<dyn Fn(&View) -> () + '_> {
        #[cfg(feature = "dom")]
        let router: RouterHandle = self.clone();
        #[allow(unused_variables)]
        let cb = move |node: &View| {
            #[cfg(feature = "dom")]
            let router = router.clone();
            #[cfg(feature = "dom")]
            let handle_click = Box::new(move |e: Event| {
                e.prevent_default();
                let element = e.current_target().unwrap().dyn_into::<Element>().unwrap();
                let href = element.get_attribute("href").unwrap();
                router.push(&href);
            }) as Box<dyn Fn(Event)>;
            #[cfg(feature = "dom")]
            node.event("click", handle_click);
        };
        Box::new(cb)
    }

    pub fn signal(&self) -> MutableSignalCloned<String> {
        self.current.signal_cloned()
    }
}

/// Represents a Single page router
#[derive(Clone)]
pub struct Router<A> {
    pub(crate) handler: RouterHandle,
    inner: matchit::Router<fn(&A) -> ViewBuilder>,
    app: PhantomData<A>,
    // on_not_found:
}

impl<A> Router<A> {
    pub fn coerce<T>(&self) -> Router<T> {
        Router {
            handler: self.handler.clone(),
            inner: Default::default(),
            app: PhantomData,
        }
    }
}

impl<A> std::fmt::Debug for Router<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("current", &self.handler)
            .finish()
    }
}

impl<A: 'static + Clone> Router<A> {
    pub fn new() -> Router<A> {
        #[allow(unused_mut)]
        let mut path = String::from("/");
        #[cfg(feature = "dom")]
        if let Some(window) = web_sys::window() {
            path = window.location().pathname().unwrap_or("/".to_string());
        }

        Self {
            handler: RouterHandle {
                current: Mutable::new(path),
            },
            inner: matchit::Router::new(),
            app: PhantomData,
        }
    }

    pub fn current_params(&self) -> HashMap<String, String> {
        let path = self.handler.current.get_cloned();
        let binding = &self.inner;
        let inner = binding.at(&path).unwrap();
        let params = &inner.params.clone();
        let params = params.iter().fold(HashMap::new(), |mut map, c| {
            map.insert(c.0.to_string(), c.1.to_string());
            map
        });
        params
    }

    /// Add a new route
    pub fn route(&mut self, path: &str, page: fn(&A) -> ViewBuilder) {
        self.inner.insert(path.to_string(), page).unwrap();
    }

    #[cfg(feature = "dom")]
    fn get_fragment() -> String {
        return web_sys::window().unwrap().location().pathname().unwrap();
    }

    pub(crate) fn render(&self, app: &A, parent: &DomType) -> View {
        let router = &self.inner;
        #[cfg(feature = "dom")]
        let current = self.handler.current.clone();
        #[cfg(feature = "dom")]
        //Hash routing forward in history and URL rewrite
        let handle_hash = Closure::wrap(Box::new(move |_evt: web_sys::Event| {
            let l: String = web_sys::window()
                .unwrap()
                .location()
                .hash()
                .unwrap()
                .chars()
                .skip(1)
                .collect();

            log::debug!("hash handle : {l}");

            let h = web_sys::window().unwrap().history().unwrap();
            h.replace_state_with_url(&JsValue::NULL, "", Some(l.as_str()))
                .unwrap();

            current.set(l.to_string());
        }) as Box<dyn Fn(_)>);
        #[cfg(feature = "dom")]
        web_sys::window()
            .unwrap()
            .set_onhashchange(Some(handle_hash.as_ref().unchecked_ref()));
        #[cfg(feature = "dom")]
        handle_hash.forget();

        #[cfg(feature = "dom")]
        let current = self.handler.current.clone();
        //Routing for navigating in history and escaping hash routes
        #[cfg(feature = "dom")]
        let handle_pop = Closure::wrap(Box::new(move |_evt: web_sys::Event| {
            let l = Self::get_fragment();

            if web_sys::window()
                .unwrap()
                .location()
                .hash()
                .unwrap()
                .chars()
                .count()
                > 0
            {
                log::debug!("hash detected");
                return ();
            }
            current.set(l.to_string());
            log::debug!("pop handle : {l}");
        }) as Box<dyn Fn(_)>);

        #[cfg(feature = "dom")]
        web_sys::window()
            .unwrap()
            .set_onpopstate(Some(handle_pop.as_ref().unchecked_ref()));

        #[cfg(feature = "dom")]
        handle_pop.forget();
        let route = (&self.handler).current.clone();

        let path = route.get_cloned();
        let current = self.inner.at(&path).unwrap();
        let page_fn = current.value;
        let builder = page_fn(&app);
        let view = builder.mount(&parent).unwrap();

        let router = router.clone();
        let app = app.clone();
        let node = parent.clone();
        let wait_for_next_route = route
            .signal_cloned()
            .map(move |route_match| {
                let match_result = router.at(&route_match).unwrap();
                let page_fn = match_result.value;

                let builder = page_fn(&app);
                let view = builder.mount(&DomType::fragment()).unwrap();
                node.replace_children_with(&view.node());
                std::mem::forget(view);
                #[cfg(feature = "dom")]
                let window = web_sys::window().unwrap();
                #[cfg(feature = "dom")]
                window
                    .history()
                    .unwrap()
                    .push_state_with_url(&JsValue::default(), "", Some(&route_match))
                    .unwrap();
                log::debug!("Router received new path: {route_match}");
            })
            .to_future();
        view.effect(wait_for_next_route);
        view
    }
}
