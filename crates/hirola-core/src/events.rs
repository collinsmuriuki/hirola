// from dominator events

use dominator::traits::StaticEvent;
use std::fmt::{Display, Error, Formatter};
use typed_html::events::EventHandler;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement, HtmlTextAreaElement};

use crate::node::Node;

macro_rules! declare_events {
    ($($name:ident : $type:ty ,)*) => {
        /// Container type for DOM events.
        pub struct Events {
            $(
                pub $name: Option<Box<dyn EventHandler<Node, $type> + Send>>,
            )*
        }

        impl Default for Events {
            fn default() -> Self {
                Events {
                    $(
                        $name: None,
                    )*
                }
            }
        }

        /// Iterate over the defined events on a DOM object.
        #[macro_export]
        macro_rules! for_events {
            ($event:ident in $events:expr => $body:block) => {
                $(
                    if let Some(ref mut $event) = $events.$name $body
                )*
            }
        }
    }
}

// TODO? these are all the "on*" attributes defined in the HTML5 standard, with
// the ones I've been unable to match to Kutuish event types commented out.
//
// This needs review.

declare_events! {
    //abort: ResourceAbortEvent,
    // autocomplete: Event,
    // autocompleteerror: Event,
    blur: Blur,
    // cancel: Event,
    // canplay: Event,
    // canplaythrough: Event,
    change: Change,
    click: Click,    // close: Event,
    contextmenu: ContextMenu,
    // cuechange: Event,
    dblclick: DoubleClick,
    drag: Drag,
    dragend: DragEnd,
    dragenter: DragEnter,
    //dragexit: DragExit,
    dragleave: DragLeave,
    dragover: DragOver,
    dragstart: DragStart,
    drop: Drop,
    // durationchange: Event,
    // emptied: Event,
    // ended: Event,
    //error: ResourceErrorEvent,
    focus: Focus,
    input: Input,
    // invalid: Event,
    keydown: KeyDown,
    //keypress: KeyPress,
    keyup: KeyUp,
    //load: ResourceLoadEvent,
    // loadeddata: Event,
    // loadedmetadata: Event,
    //loadstart: LoadStartEvent,
    mousedown: MouseDown,
    mouseenter: MouseEnter,
    mouseleave: MouseLeave,
    mousemove: MouseMove,
    // mouseout: MouseOut,
    // mouseover: MouseOver,
    mouseup: MouseUp,
    // mousewheel: MouseWheelEvent,
    // pause: Event,
    // play: Event,
    // playing: Event,
    // progress: ProgressEvent,
    // ratechange: Event,
    // reset: Event,
    resize: Resize,
    scroll: Scroll,
    // seeked: Event,
    // seeking: Event,
    // select: Event,
    // show: Event,
    // sort: Event,
    // stalled: Event,
    // submit: Submit,
    // suspend: Event,
    // timeupdate: Event,
    // toggle: Event,
    // volumechange: Event,
    // waiting: Event,
}

impl Display for Events {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), Error> {
        Ok(())
    }
}

// /// Wrapper type for closures as event handlers.
// pub struct EFn<F, E>(Option<F>, PhantomData<E>);

// impl<F, E> EFn<F, E>
// where
//     F: FnMut(E) + 'static + Send,
// {
//     pub fn new(f: F) -> Self {
//         EFn(Some(f), PhantomData)
//     }
// }

// impl<F, E> From<F> for Box<dyn EventHandler<Kutuish, E> + Send>
// where
//     F: FnMut(E) + 'static + Send,
//     E: StaticEvent + 'static + Send,
// {
//     fn from(f: F) -> Self {
//         Box::new(EFn::new(f))
//     }
// }

// impl<F, E> EventHandler<Kutuish, E> for EFn<F, E>
// where
//     F: FnMut(E) + 'static + Send,
//     E: StaticEvent + std::marker::Send + 'static,
// {
//     fn attach(&mut self, target: &mut <Kutuish as OutputType>::EventTarget) {
//         let mut handler = self.0.take().unwrap();
//         (target.event)(&mut handler);

//     }

//     fn render(&self) -> Option<String> {
//         None
//     }
// }

macro_rules! make_event {
    ($name:ident, $type:literal => $event:path) => {
        pub struct $name {
            event: $event,
        }

        impl StaticEvent for $name {
            const EVENT_TYPE: &'static str = $type;

            #[inline]
            fn unchecked_from_event(event: web_sys::Event) -> Self {
                Self {
                    event: event.unchecked_into(),
                }
            }
        }

        impl $name {
            #[inline]
            pub fn prevent_default(&self) {
                self.event.prevent_default();
            }

            #[inline]
            pub fn target(&self) -> Option<EventTarget> {
                self.event.target()
            }

            #[inline]
            pub fn dyn_target<A>(&self) -> Option<A>
            where
                A: JsCast,
            {
                self.target()?.dyn_into().ok()
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

macro_rules! make_mouse_event {
    ($name:ident, $type:literal) => {
        make_event!($name, $type => web_sys::MouseEvent);

        impl $name {
            #[inline] pub fn x(&self) -> i32 { self.event.client_x() }
            #[inline] pub fn y(&self) -> i32 { self.event.client_y() }

            #[inline] pub fn screen_x(&self) -> i32 { self.event.screen_x() }
            #[inline] pub fn screen_y(&self) -> i32 { self.event.screen_y() }

            #[inline] pub fn ctrl_key(&self) -> bool { self.event.ctrl_key() || self.event.meta_key() }
            #[inline] pub fn shift_key(&self) -> bool { self.event.shift_key() }
            #[inline] pub fn alt_key(&self) -> bool { self.event.alt_key() }

            #[inline] pub fn mouse_x(&self) -> i32 { self.event.client_x() }
            #[inline] pub fn mouse_y(&self) -> i32 { self.event.client_y() }

            pub fn button(&self) -> MouseButton {
                match self.event.button() {
                    0 => MouseButton::Left,
                    1 => MouseButton::Middle,
                    2 => MouseButton::Right,
                    3 => MouseButton::Button4,
                    4 => MouseButton::Button5,
                    _ => unreachable!("Unexpected MouseEvent.button value"),
                }
            }
        }
    };
}

macro_rules! make_keyboard_event {
    ($name:ident, $type:literal) => {
        make_event!($name, $type => web_sys::KeyboardEvent);

        impl $name {
            // TODO return enum or something
            #[inline] pub fn key(&self) -> String { self.event.key() }

            #[inline] pub fn ctrl_key(&self) -> bool { self.event.ctrl_key() || self.event.meta_key() }
            #[inline] pub fn shift_key(&self) -> bool { self.event.shift_key() }
            #[inline] pub fn alt_key(&self) -> bool { self.event.alt_key() }
        }
    };
}

macro_rules! make_focus_event {
    ($name:ident, $type:literal) => {
        make_event!($name, $type => web_sys::FocusEvent);
    };
}

macro_rules! make_drag_event {
    ($name:ident, $type:literal) => {
        make_event!($name, $type => web_sys::DragEvent);

        impl $name {
            #[inline] pub fn data_transfer(&self) -> Option<web_sys::DataTransfer> { self.event.data_transfer() }
        }
    };
}

make_mouse_event!(Click, "click");
make_mouse_event!(MouseDown, "mousedown");
make_mouse_event!(MouseUp, "mouseup");
make_mouse_event!(MouseMove, "mousemove");
make_mouse_event!(MouseEnter, "mouseenter");
make_mouse_event!(MouseLeave, "mouseleave");
make_mouse_event!(DoubleClick, "dblclick");
make_mouse_event!(ContextMenu, "contextmenu");

make_keyboard_event!(KeyDown, "keydown");
make_keyboard_event!(KeyUp, "keyup");

make_focus_event!(Focus, "focus");
make_focus_event!(Blur, "blur");

make_drag_event!(DragStart, "dragstart");
make_drag_event!(Drag, "drag");
make_drag_event!(DragEnd, "dragend");
make_drag_event!(DragOver, "dragover");
make_drag_event!(DragEnter, "dragenter");
make_drag_event!(DragLeave, "dragleave");
make_drag_event!(Drop, "drop");

make_event!(Scroll, "scroll" => web_sys::Event);
make_event!(Resize, "resize" => web_sys::UiEvent);
make_event!(Input, "input" => web_sys::InputEvent);

impl Input {
    // TODO should this work on other types as well ?
    pub fn value(&self) -> Option<String> {
        let target = self.target()?;

        if let Some(target) = target.dyn_ref::<HtmlInputElement>() {
            // TODO check the <input> element's type ?
            Some(target.value())
        } else if let Some(target) = target.dyn_ref::<HtmlTextAreaElement>() {
            Some(target.value())
        } else {
            None
        }
    }
}

make_event!(Change, "change" => web_sys::Event);

// TODO add in a value method as well, the same as Input::value
impl Change {
    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement
    pub fn checked(&self) -> Option<bool> {
        let target = self.dyn_target::<HtmlInputElement>()?;

        match target.type_().as_str() {
            "checkbox" | "radio" => Some(target.checked()),
            _ => None,
        }
    }
}
