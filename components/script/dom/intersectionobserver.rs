/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use base::cross_process_instant::CrossProcessInstant;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};
use style::context::QuirksMode;
use style::parser::{Parse, ParserContext};
use style::stylesheets::{CssRuleType, Origin};
use style_traits::{ParsingMode, ToCss};
use url::Url;

use super::bindings::codegen::Bindings::IntersectionObserverBinding::{
    IntersectionObserverCallback, IntersectionObserverMethods,
};
use super::bindings::codegen::UnionTypes::{DoubleOrDoubleSequence, ElementOrDocument};
use super::bindings::num::Finite;
use super::bindings::utils::to_frozen_array;
use super::intersectionobserverrootmargin::IntersectionObserverRootMargin;
use super::types::{Element, IntersectionObserverEntry};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IntersectionObserverBinding::IntersectionObserverInit;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};
/// The Intersection Observer interface
///
/// > The IntersectionObserver interface can be used to observe changes in the intersection
/// > of an intersection root and one or more target Elements.
///
/// <https://w3c.github.io/IntersectionObserver/#intersection-observer-interface>
#[dom_struct]
pub(crate) struct IntersectionObserver {
    reflector_: Reflector,

    /// > This callback will be invoked when there are changes to a target’s intersection
    /// > with the intersection root, as per the processing model.
    /// <https://w3c.github.io/IntersectionObserver/#intersection-observer-callback>
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<IntersectionObserverCallback>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-queuedentries-slot>
    queued_entries: DomRefCell<Vec<DomRoot<IntersectionObserverEntry>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observationtargets-slot>
    observation_targets: DomRefCell<Vec<Dom<Element>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin-slot>
    #[no_trace]
    #[ignore_malloc_size_of = "Defined in style"]
    root_margin: RefCell<IntersectionObserverRootMargin>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-scrollmargin-slot>
    #[no_trace]
    #[ignore_malloc_size_of = "Defined in style"]
    scroll_margin: RefCell<IntersectionObserverRootMargin>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-thresholds-slot>
    thresholds: RefCell<Vec<Finite<f64>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-delay-slot>
    delay: Cell<i32>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-trackvisibility-slot>
    track_visibility: Cell<bool>,
}

impl IntersectionObserver {
    pub(crate) fn new_inherited(
        callback: Rc<IntersectionObserverCallback>,
        root_margin: IntersectionObserverRootMargin,
        scroll_margin: IntersectionObserverRootMargin,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            callback,
            queued_entries: Default::default(),
            observation_targets: Default::default(),
            root_margin: RefCell::new(root_margin),
            scroll_margin: RefCell::new(scroll_margin),
            thresholds: Default::default(),
            delay: Default::default(),
            track_visibility: Default::default(),
        }
    }

    /// <https://w3c.github.io/IntersectionObserver/#initialize-new-intersection-observer>
    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Self>> {
        // Step 3.
        // > Attempt to parse a margin from options.rootMargin. If a list is returned,
        // > set this’s internal [[rootMargin]] slot to that. Otherwise, throw a SyntaxError exception.
        let root_margin = if let Ok(margin) = parse_a_margin(init.rootMargin.as_ref()) {
            margin
        } else {
            return Err(Error::Syntax);
        };

        // Step 4.
        // > Attempt to parse a margin from options.scrollMargin. If a list is returned,
        // > set this’s internal [[scrollMargin]] slot to that. Otherwise, throw a SyntaxError exception.
        let scroll_margin = if let Ok(margin) = parse_a_margin(init.scrollMargin.as_ref()) {
            margin
        } else {
            return Err(Error::Syntax);
        };

        // Step 1 and step 2, 3, 4 setter
        // > 1. Let this be a new IntersectionObserver object
        // > 2. Set this’s internal [[callback]] slot to callback.
        // > 3. ... set this’s internal [[rootMargin]] slot to that.
        // > 4. ,.. set this’s internal [[scrollMargin]] slot to that.
        let observer = reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(callback, root_margin, scroll_margin)),
            window,
            proto,
            can_gc,
        );

        observer.init_observer(init)?;

        Ok(observer)
    }

    /// Step 5-13 of <https://w3c.github.io/IntersectionObserver/#initialize-new-intersection-observer>
    fn init_observer(&self, init: &IntersectionObserverInit) -> Fallible<()> {
        // Step 5
        // > Let thresholds be a list equal to options.threshold.
        //
        // Non-sequence value should be converted into Vec.
        // Default value of thresholds is [0].
        let mut thresholds = match &init.threshold {
            Some(DoubleOrDoubleSequence::Double(num)) => vec![*num],
            Some(DoubleOrDoubleSequence::DoubleSequence(sequence)) => sequence.clone(),
            None => vec![Finite::wrap(0.)],
        };

        // Step 6
        // > If any value in thresholds is less than 0.0 or greater than 1.0, throw a RangeError exception.
        thresholds
            .iter()
            .try_for_each(|num| match **num < 0.0 || **num > 1.0 {
                true => Err(Error::Range(
                    "Value in thresholds should not be less than 0.0 or greater than 1.0"
                        .to_owned(),
                )),
                false => Ok(()),
            })?;

        // Step 7
        // > Sort thresholds in ascending order.
        thresholds.sort_by(|lhs, rhs| lhs.partial_cmp(&**rhs).unwrap());

        // Step 8
        // > If thresholds is empty, append 0 to thresholds.
        if thresholds.is_empty() {
            thresholds.push(Finite::wrap(0.));
        }

        // Step 9
        // > The thresholds attribute getter will return this sorted thresholds list.
        //
        // Set this’s internal [[thresholds]] slot to the sorted thresholds list
        // and getter will return the internal [[thresholds]] slot.
        self.thresholds.replace(thresholds);

        // Step 10
        // > Let delay be the value of options.delay.
        //
        // Default value of delay is 0.
        let mut delay = init.delay.unwrap_or(0);

        // Step 11
        // > If options.trackVisibility is true and delay is less than 100, set delay to 100.
        if init.trackVisibility {
            delay = delay.min(100);
        }

        // Step 12
        // > Set this’s internal [[delay]] slot to options.delay to delay.
        self.delay.set(delay);

        // Step 13
        // > Set this’s internal [[trackVisibility]] slot to options.trackVisibility.
        self.track_visibility.set(init.trackVisibility);

        Ok(())
    }
}

impl IntersectionObserverMethods<crate::DomTypeHolder> for IntersectionObserver {
    /// > The root provided to the IntersectionObserver constructor, or null if none was provided.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-root>
    fn GetRoot(&self) -> Option<ElementOrDocument> {
        None
    }

    /// > Offsets applied to the root intersection rectangle, effectively growing or
    /// > shrinking the box that is used to calculate intersections. These offsets are only
    /// > applied when handling same-origin-domain targets; for cross-origin-domain targets
    /// > they are ignored.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin>
    fn RootMargin(&self) -> DOMString {
        DOMString::from_string(self.root_margin.borrow().to_css_string())
    }

    /// > Offsets are applied to scrollports on the path from intersection root to target,
    /// > effectively growing or shrinking the clip rects used to calculate intersections.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-scrollmargin>
    fn ScrollMargin(&self) -> DOMString {
        DOMString::from_string(self.scroll_margin.borrow().to_css_string())
    }

    /// > A list of thresholds, sorted in increasing numeric order, where each threshold
    /// > is a ratio of intersection area to bounding box area of an observed target.
    /// > Notifications for a target are generated when any of the thresholds are crossed
    /// > for that target. If no options.threshold was provided to the IntersectionObserver
    /// > constructor, or the sequence is empty, the value of this attribute will be `[0]`.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-thresholds>
    fn Thresholds(&self, context: JSContext, retval: MutableHandleValue) {
        to_frozen_array(&self.thresholds.borrow(), context, retval);
    }

    /// > A number indicating the minimum delay in milliseconds between notifications from
    /// > this observer for a given target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-delay>
    fn Delay(&self) -> i32 {
        self.delay.get()
    }

    /// > A boolean indicating whether this IntersectionObserver will track changes in a target’s visibility.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-trackvisibility>
    fn TrackVisibility(&self) -> bool {
        self.track_visibility.get()
    }

    /// > Run the observe a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observe>
    fn Observe(&self, _target: &Element) {}

    /// > Run the unobserve a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-unobserve>
    fn Unobserve(&self, _target: &Element) {}

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-disconnect>
    fn Disconnect(&self) {}

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-takerecords>
    fn TakeRecords(&self) -> Vec<DomRoot<IntersectionObserverEntry>> {
        vec![]
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-intersectionobserver>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
    ) -> Fallible<DomRoot<IntersectionObserver>> {
        Self::new(window, proto, callback, init, can_gc)
    }
}

/// <https://w3c.github.io/IntersectionObserver/#intersectionobserverregistration>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct IntersectionObserverRegistration {
    observer: DomRoot<IntersectionObserver>,
    previous_threshold_index: Cell<i32>,
    previous_is_intersecting: Cell<bool>,
    #[no_trace]
    last_update_time: Cell<CrossProcessInstant>,
    previous_is_visible: Cell<bool>,
}

/// <https://w3c.github.io/IntersectionObserver/#parse-a-margin>
fn parse_a_margin(value: Option<&DOMString>) -> Result<IntersectionObserverRootMargin, ()> {
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverinit-rootmargin> &&
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverinit-scrollmargin>
    // > ... defaulting to "0px".
    let value = match value {
        Some(str) => str.str(),
        _ => "0px",
    };

    // Create necessary style ParserContext and utilize stylo's IntersectionObserverRootMargin
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);

    let url = Url::parse("about:blank").unwrap().into();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    );

    match parser.parse_entirely(|p| IntersectionObserverRootMargin::parse(&context, p)) {
        Ok(margin) => Ok(margin),
        _ => Err(()),
    }
}
