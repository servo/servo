/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

macro_rules! event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(
            script::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull<TypeHolder>,
            $event_type,
            $getter,
            $setter,
            set_event_handler_common
        );
    )
);

macro_rules! define_event_handler(
    ($handler: ty, $event_type: ident, $getter: ident, $setter: ident, $setter_fn: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            use script::dom::bindings::inheritance::Castable;
            use script::dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget<TypeHolder>>();
            eventtarget.get_event_handler_common(stringify!($event_type))
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            use script::dom::bindings::inheritance::Castable;
            use script::dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget<TypeHolder>>();
            eventtarget.$setter_fn(stringify!($event_type), listener)
        }
    )
);

