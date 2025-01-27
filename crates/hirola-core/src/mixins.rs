//! ## Mixins
//! Hirola aims to be extensible and follow DRY principles.
//! Here is an example of a mixin
//! ```rust,no_run
//! use hirola::prelude::*;
//! use web_sys::Element;
//! // Mixin that controls tailwind opacity based on a bool signal
//! fn opacity<'a>(signal: &'a Mutable<bool>) -> Box<dyn Fn(&Dom) -> () + 'a> {
//!    let cb = move |dom: &Dom| {
//!        let node = dom.node().clone();
//!        let element = node.unchecked_into::<Element>();
//!        if signal.get() {
//!            element.class_list().add_1("opacity-100").unwrap();
//!            element.class_list().remove_1("opacity-0").unwrap();
//!        } else {
//!            element.class_list().add_1("opacity-0").unwrap();
//!            element.class_list().remove_1("opacity-100").unwrap();
//!        }
//!    };
//!    Box::new(cb)
//! }
//!
//! fn mixin_demo() -> Dom {
//!    let is_shown = Mutable::new(true);
//!    let toggle = is_shown.callback(|show| {
//!         let current = show.get();
//!         *show.lock_mut() = !current;
//!    });
//!    html! {
//!        <div
//!            class="h-screen flex flex-col items-center justify-center transition-all ease-in-out delay-1000">
//!            <div
//!                class="h-64 w-64 block bg-blue-900 rounded-md"
//!                mixin:identity=&opacity(&is_shown)/>
//!            <button
//!                class="bg-gray-200 mt-4 font-bold py-2 px-4 rounded"
//!                on:click=toggle>
//!                "Click Me"
//!            </button>
//!        </div>
//!    }
//! }
//! fn main() {
//!
//! }
//! ```
use crate::dom::Dom;
use futures_signals::signal::{Signal, SignalExt};
use std::fmt::Display;
#[cfg(feature = "dom")]
use wasm_bindgen::JsCast;
#[cfg(feature = "dom")]
use web_sys::Element;

pub trait Mixin<Target> {
    fn mixin(&self, node: &Dom);
}

/// Unbound mixin in the form of `Fn(&Dom)`
///
/// ## Example
/// ```rust,no_run
/// use hirola::prelude::*;
/// fn counter() -> Dom {
///     html! {
///         <span mixin:identity=&raw_text("Hello Counter!") />
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Identity;

impl<T> Mixin<Identity> for T
where
    T: Fn(&Dom),
{
    fn mixin(&self, node: &Dom) {
        (&self)(node);
    }
}

/// A mixin that allows adding raw html
/// Note: This is a security risk if the string to be inserted might contain potentially malicious content.
/// sanitize the content before it is inserted.
/// See more: https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML
#[allow(unused_variables)]
pub fn raw_html<'a>(text: &'a str) -> Box<dyn Fn(&Dom) -> () + 'a> {
    let cb = move |node: &Dom| {
        #[cfg(feature = "dom")]
        {
            let element = node.node().as_ref().clone().unchecked_into::<Element>();
            element.set_inner_html(text);
        }
    };
    Box::new(cb)
}

/// A mixin that allows adding non-signal text
#[allow(unused_variables)]
pub fn raw_text<'a>(text: &'a str) -> Box<dyn Fn(&Dom) + 'a> {
    let cb = move |dom: &Dom| {
        #[cfg(feature = "dom")]
        {
            dom.node().node.set_text_content(Some(&format!("{text}")));
        }
    };
    Box::new(cb)
}

/// Mixin that adds text to a dom node
#[allow(unused_variables)]
pub fn text<T, S>(text: &S) -> Box<dyn Fn(&Dom)>
where
    T: Display + 'static,
    S: Signal<Item = T> + SignalExt + Clone + 'static,
{
    let signal = text.clone();
    
    let cb = move |_node: &Dom| {
        #[cfg(feature = "dom")]
        {
            use std::future::ready;
            let element = _node.node().as_ref().clone().unchecked_into::<Element>();
            let signal = signal.clone();
            let future = signal.for_each(move |value| {
                element.set_text_content(Some(&format!("{}", value)));
                ready(())
            });
            _node.effect(future);
        }
    };
    Box::new(cb)
}
