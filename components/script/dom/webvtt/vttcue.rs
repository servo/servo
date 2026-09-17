/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::local_name;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_webvtt::cue::settings::{
    WebVttCue, WebVttCueSize, WebVttLineAlignment, WebVttLineAndPositionSetting,
    WebVttPositionAlignment, WebVttSnapToLines, WebVttTextAlignment, WebVttWritingDirection,
};
use servo_webvtt::cue::text::{
    WebVTTNodeObject, WebVTTNodeObjectIterable, WebVTTNodeObjectIteratorDirection,
    WebVTTNodeObjectKind, webvtt_cue_text_parsing_rules,
};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::VTTCueBinding::{
    self, AlignSetting, AutoKeyword, DirectionSetting, LineAlignSetting, PositionAlignSetting,
    VTTCueMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::node::Node;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::vttregion::VTTRegion;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct VTTCue {
    texttrackcue: TextTrackCue,
    region: DomRefCell<Option<Dom<VTTRegion>>>,
    vertical: Cell<DirectionSetting>,
    snap_to_lines: Cell<bool>,
    line: DomRefCell<LineAndPositionSetting>,
    line_align: Cell<LineAlignSetting>,
    position: DomRefCell<LineAndPositionSetting>,
    position_align: Cell<PositionAlignSetting>,
    size: Cell<f64>,
    align: Cell<AlignSetting>,
    text: DomRefCell<DOMString>,
}

impl VTTCue {
    #[expect(clippy::too_many_arguments)]
    fn new_inherited(
        start_time: f64,
        end_time: f64,
        text: DOMString,
        id: DOMString,
        vertical: DirectionSetting,
        snap_to_lines: bool,
        line: LineAndPositionSetting,
        line_align: LineAlignSetting,
        position: LineAndPositionSetting,
        position_align: PositionAlignSetting,
        size: f64,
        align: AlignSetting,
        track: Option<&TextTrack>,
    ) -> Self {
        VTTCue {
            texttrackcue: TextTrackCue::new_inherited(id, start_time, end_time, track),
            text: DomRefCell::new(text),
            region: DomRefCell::new(None),
            vertical: Cell::new(vertical),
            snap_to_lines: Cell::new(snap_to_lines),
            line: DomRefCell::new(line),
            line_align: Cell::new(line_align),
            position: DomRefCell::new(position),
            position_align: Cell::new(position_align),
            size: Cell::new(size),
            align: Cell::new(align),
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        start_time: f64,
        end_time: f64,
        text: DOMString,
        id: DOMString,
        vertical: DirectionSetting,
        snap_to_lines: bool,
        line: LineAndPositionSetting,
        line_align: LineAlignSetting,
        position: LineAndPositionSetting,
        position_align: PositionAlignSetting,
        size: f64,
        align: AlignSetting,
        track: Option<&TextTrack>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self::new_inherited(
                start_time,
                end_time,
                text,
                id,
                vertical,
                snap_to_lines,
                line,
                line_align,
                position,
                position_align,
                size,
                align,
                track,
            )),
            window,
            proto,
        )
    }

    pub(crate) fn create_from_vtt(
        cx: &mut JSContext,
        vtt_cue: WebVttCue,
        window: &Window,
        track: Option<&TextTrack>,
    ) -> DomRoot<VTTCue> {
        Self::new(
            cx,
            window,
            None,
            vtt_cue.start_time,
            vtt_cue.end_time,
            vtt_cue.text.into(),
            vtt_cue.identifier.into(),
            vtt_cue.writing_direction.convert(),
            vtt_cue.snap_to_lines.convert(),
            vtt_cue.line.convert(),
            vtt_cue.line_alignment.convert(),
            vtt_cue.position.convert(),
            vtt_cue.position_alignment.convert(),
            vtt_cue.size.convert(),
            vtt_cue.text_alignment.convert(),
            track,
        )
    }

    /// <https://w3c.github.io/webvtt/#dom-construction-rules>
    fn webvtt_cue_text_dom_construction_rules(
        &self,
        cx: &mut JSContext,
        root_object: Rc<WebVTTNodeObject>,
    ) -> DomRoot<DocumentFragment> {
        assert!(root_object.kind == WebVTTNodeObjectKind::List);

        let global = self.global();
        let window = global.as_window();
        // > The ownerDocument attribute of all nodes in the DOM
        // > tree must be set to the given document owner.
        let document = window.Document();
        let root = DocumentFragment::new(cx, &document);
        // > To convert a list of WebVTT Node Objects to a DOM tree for Document owner,
        // > user agents must create a tree of DOM nodes that is isomorphous
        // > to the tree of WebVTT Node Objects,
        // > with the following mapping of WebVTT Node Objects to DOM nodes:
        let mut current = DomRoot::upcast::<Node>(root.clone());
        for node_traversal in root_object.into_iter() {
            match node_traversal {
                WebVTTNodeObjectIteratorDirection::NewChild(new_child) => {
                    let new_tag_name = match &new_child.kind {
                        WebVTTNodeObjectKind::Italic => "i",
                        WebVTTNodeObjectKind::Bold => "b",
                        WebVTTNodeObjectKind::Underline => "u",
                        WebVTTNodeObjectKind::Ruby => "ruby",
                        WebVTTNodeObjectKind::RubyText => "rt",
                        WebVTTNodeObjectKind::Class |
                        WebVTTNodeObjectKind::Voice(_) |
                        WebVTTNodeObjectKind::Language => "span",
                        WebVTTNodeObjectKind::Text(text) => {
                            // > Text node whose data is the value of the WebVTT Text Object.
                            let text = document.CreateTextNode(cx, DOMString::from(text.as_ref()));
                            if current.AppendChild(cx, text.upcast()).is_err() {
                                unreachable!("Must always be able to append");
                            }
                            continue;
                        },
                        WebVTTNodeObjectKind::Timestamp(timestamp) => {
                            // > ProcessingInstruction node whose target is "timestamp"
                            // > and whose data is a WebVTT timestamp representing the
                            // > value of the WebVTT Timestamp Object,
                            // > with all optional components included,
                            // > with one leading zero if the hours component is less than ten,
                            // > and with no leading zeros otherwise.
                            let new_child = document
                                .CreateProcessingInstruction(
                                    cx,
                                    DOMString::from_static("timestamp"),
                                    format!("{timestamp}").into(),
                                )
                                .expect("Must always be able to create processing instruction");
                            if current.AppendChild(cx, new_child.upcast()).is_err() {
                                unreachable!("Must always be able to append");
                            }
                            current = DomRoot::upcast(new_child);
                            continue;
                        },
                        WebVTTNodeObjectKind::List => {
                            let new_child = document.CreateDocumentFragment(cx);
                            if current.AppendChild(cx, new_child.upcast()).is_err() {
                                unreachable!("Must always be able to append");
                            }
                            current = DomRoot::upcast(new_child);
                            continue;
                        },
                    };
                    let new_child_element = document.create_element(cx, new_tag_name);
                    match &new_child.kind {
                        WebVTTNodeObjectKind::Language => {
                            // > HTML span element with a lang attribute set to
                            // > the WebVTT Language Object’s applicable language.
                            new_child_element.set_attribute(
                                cx,
                                &local_name!("lang"),
                                new_child.applicable_language.clone().into(),
                            );
                        },
                        WebVTTNodeObjectKind::Voice(title) => {
                            // > HTML span element with a title attribute set to
                            // > the WebVTT Voice Object’s value.
                            new_child_element.set_attribute(
                                cx,
                                &local_name!("title"),
                                title.clone().into(),
                            );
                        },
                        _ => {},
                    }
                    // > HTML elements created as part of the mapping described above must
                    // > have their namespaceURI set to the HTML namespace,
                    // > use the appropriate IDL interface as defined in the HTML specification,
                    // > and, if the corresponding WebVTT Internal Node Object has any applicable classes,
                    // > must have a class attribute set to the string obtained by concatenating all those classes,
                    // > each separated from the next by a single U+0020 SPACE character.
                    if !new_child.applicable_classes.is_empty() {
                        new_child_element.set_attribute(
                            cx,
                            &local_name!("class"),
                            new_child.applicable_classes.join("\u{0020}").into(),
                        );
                    }
                    if current.AppendChild(cx, new_child_element.upcast()).is_err() {
                        unreachable!("Must always be able to append");
                    }
                    current = DomRoot::upcast(new_child_element);
                },
                WebVTTNodeObjectIteratorDirection::BackToParent => {
                    current = current.GetParentNode().expect("Must always have a parent");
                },
            }
        }
        root
    }
}

impl VTTCueMethods<crate::DomTypeHolder> for VTTCue {
    /// <https://w3c.github.io/webvtt/#dom-vttcue-vttcue>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        start_time: Finite<f64>,
        end_time: Finite<f64>,
        text: DOMString,
    ) -> Fallible<DomRoot<Self>> {
        // Step 3. If the value of the endTime argument is negative Infinity or a Not-a-Number (NaN) value,
        // then throw a TypeError exception.
        // Otherwise, let cue’s text track cue end time be the value of the endTime argument.
        if (end_time.is_infinite() && end_time.is_sign_negative()) || end_time.is_nan() {
            return Err(Error::Type(
                c"End time is negative Infinity or Not-a-Number".to_owned(),
            ));
        }
        // Step 1. Create a new WebVTT cue. Let cue be that WebVTT cue.
        // Step 16. Return the VTTCue object representing cue.
        Ok(VTTCue::new(
            cx,
            window,
            proto,
            // Step 2. Let cue’s text track cue start time be the value of the startTime argument.
            *start_time,
            *end_time,
            // Step 4. Let cue’s cue text be the value of the text argument,
            // and let the rules for extracting the chapter title be the
            // WebVTT rules for extracting the chapter title.
            // TODO: Extract title
            text,
            // Step 5. Let cue’s text track cue identifier be the empty string.
            Default::default(),
            // Step 8. Let cue’s WebVTT cue writing direction be horizontal.
            DirectionSetting::_empty,
            // Step 9. Let cue’s WebVTT cue snap-to-lines flag be true.
            true,
            // Step 10. Let cue’s WebVTT cue line be auto.
            LineAndPositionSetting::Auto,
            // Step 11. Let cue’s WebVTT cue line alignment be start alignment.
            LineAlignSetting::Start,
            // Step 12. Let cue’s WebVTT cue position be auto.
            LineAndPositionSetting::Auto,
            // Step 13. Let cue’s WebVTT cue position alignment be auto.
            PositionAlignSetting::Auto,
            // Step 14. Let cue’s WebVTT cue size be 100.
            100.,
            // Step 15. Let cue’s WebVTT cue text alignment be center alignment.
            AlignSetting::Center,
            None,
        ))
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-region>
    fn GetRegion(&self) -> Option<DomRoot<VTTRegion>> {
        self.region
            .borrow()
            .as_ref()
            .map(|r| DomRoot::from_ref(&**r))
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-region>
    fn SetRegion(&self, value: Option<&VTTRegion>) {
        *self.region.borrow_mut() = value.map(Dom::from_ref)
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-vertical>
    fn Vertical(&self) -> DirectionSetting {
        self.vertical.get()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-vertical>
    fn SetVertical(&self, value: DirectionSetting) {
        self.vertical.set(value);
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-snaptolines>
    fn SnapToLines(&self) -> bool {
        self.snap_to_lines.get()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-snaptolines>
    fn SetSnapToLines(&self, value: bool) {
        self.snap_to_lines.set(value)
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-line>
    fn Line(&self) -> VTTCueBinding::LineAndPositionSetting {
        VTTCueBinding::LineAndPositionSetting::from(self.line.borrow().clone())
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-line>
    fn SetLine(&self, value: VTTCueBinding::LineAndPositionSetting) {
        *self.line.borrow_mut() = value.into();
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-linealign>
    fn LineAlign(&self) -> LineAlignSetting {
        self.line_align.get()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-linealign>
    fn SetLineAlign(&self, value: LineAlignSetting) {
        self.line_align.set(value);
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-position>
    fn Position(&self) -> VTTCueBinding::LineAndPositionSetting {
        VTTCueBinding::LineAndPositionSetting::from(self.position.borrow().clone())
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-position>
    fn SetPosition(&self, value: VTTCueBinding::LineAndPositionSetting) -> ErrorResult {
        if let VTTCueBinding::LineAndPositionSetting::Double(x) = value &&
            (*x < 0_f64 || *x > 100_f64)
        {
            return Err(Error::IndexSize(None));
        }

        *self.position.borrow_mut() = value.into();
        Ok(())
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-positionalign>
    fn PositionAlign(&self) -> PositionAlignSetting {
        self.position_align.get()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-positionalign>
    fn SetPositionAlign(&self, value: PositionAlignSetting) {
        self.position_align.set(value);
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-size>
    fn Size(&self) -> Finite<f64> {
        Finite::wrap(self.size.get())
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-size>
    fn SetSize(&self, value: Finite<f64>) -> ErrorResult {
        if *value < 0_f64 || *value > 100_f64 {
            return Err(Error::IndexSize(None));
        }

        self.size.set(*value);
        Ok(())
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-align>
    fn Align(&self) -> AlignSetting {
        self.align.get()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-align>
    fn SetAlign(&self, value: AlignSetting) {
        self.align.set(value);
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-text>
    fn Text(&self) -> DOMString {
        self.text.borrow().clone()
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-text>
    fn SetText(&self, value: DOMString) {
        *self.text.borrow_mut() = value;
    }

    /// <https://w3c.github.io/webvtt/#dom-vttcue-getcueashtml>
    fn GetCueAsHTML(&self, cx: &mut JSContext) -> DomRoot<DocumentFragment> {
        // > The getCueAsHTML() method must convert the cue text to a DocumentFragment
        // > for the responsible document specified by the entry settings object
        // > by applying the WebVTT cue text DOM construction rules to the result
        // > of applying the WebVTT cue text parsing rules to the cue text.
        self.webvtt_cue_text_dom_construction_rules(
            cx,
            webvtt_cue_text_parsing_rules(&self.text.borrow().str(), None),
        )
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
enum LineAndPositionSetting {
    Double(f64),
    Auto,
}

impl From<VTTCueBinding::LineAndPositionSetting> for LineAndPositionSetting {
    fn from(value: VTTCueBinding::LineAndPositionSetting) -> Self {
        match value {
            VTTCueBinding::LineAndPositionSetting::Double(x) => LineAndPositionSetting::Double(*x),
            VTTCueBinding::LineAndPositionSetting::AutoKeyword(_) => LineAndPositionSetting::Auto,
        }
    }
}

impl From<LineAndPositionSetting> for VTTCueBinding::LineAndPositionSetting {
    fn from(value: LineAndPositionSetting) -> Self {
        match value {
            LineAndPositionSetting::Double(x) => {
                VTTCueBinding::LineAndPositionSetting::Double(Finite::wrap(x))
            },
            LineAndPositionSetting::Auto => {
                VTTCueBinding::LineAndPositionSetting::AutoKeyword(AutoKeyword::Auto)
            },
        }
    }
}

impl Convert<LineAndPositionSetting> for WebVttLineAndPositionSetting {
    fn convert(self) -> LineAndPositionSetting {
        match self {
            WebVttLineAndPositionSetting::Double(x) => LineAndPositionSetting::Double(x),
            WebVttLineAndPositionSetting::Auto => LineAndPositionSetting::Auto,
        }
    }
}

impl Convert<DirectionSetting> for WebVttWritingDirection {
    fn convert(self) -> DirectionSetting {
        match self {
            WebVttWritingDirection::Horizontal => DirectionSetting::_empty,
            WebVttWritingDirection::VerticalGrowingLeft => DirectionSetting::Rl,
            WebVttWritingDirection::VerticalGrowingRight => DirectionSetting::Lr,
        }
    }
}

impl Convert<AlignSetting> for WebVttTextAlignment {
    fn convert(self) -> AlignSetting {
        match self {
            WebVttTextAlignment::Start => AlignSetting::Start,
            WebVttTextAlignment::Center => AlignSetting::Center,
            WebVttTextAlignment::End => AlignSetting::End,
            WebVttTextAlignment::Left => AlignSetting::Left,
            WebVttTextAlignment::Right => AlignSetting::Right,
        }
    }
}

impl Convert<LineAlignSetting> for WebVttLineAlignment {
    fn convert(self) -> LineAlignSetting {
        match self {
            WebVttLineAlignment::Start => LineAlignSetting::Start,
            WebVttLineAlignment::Center => LineAlignSetting::Center,
            WebVttLineAlignment::End => LineAlignSetting::End,
        }
    }
}

impl Convert<PositionAlignSetting> for WebVttPositionAlignment {
    fn convert(self) -> PositionAlignSetting {
        match self {
            WebVttPositionAlignment::LineLeft => PositionAlignSetting::Line_left,
            WebVttPositionAlignment::Center => PositionAlignSetting::Center,
            WebVttPositionAlignment::LineRight => PositionAlignSetting::Line_right,
            WebVttPositionAlignment::Auto => PositionAlignSetting::Auto,
        }
    }
}

impl Convert<bool> for WebVttSnapToLines {
    fn convert(self) -> bool {
        match self {
            WebVttSnapToLines::Yes => true,
            WebVttSnapToLines::No => false,
        }
    }
}

impl Convert<f64> for WebVttCueSize {
    fn convert(self) -> f64 {
        self.0
    }
}
