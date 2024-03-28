/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VTTCueBinding::{
    self, AlignSetting, AutoKeyword, DirectionSetting, LineAlignSetting, PositionAlignSetting,
    VTTCueMethods,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::globalscope::GlobalScope;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::vttregion::VTTRegion;
use crate::dom::window::Window;

#[dom_struct]
pub struct VTTCue {
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
    pub fn new_inherited(start_time: f64, end_time: f64, text: DOMString) -> Self {
        VTTCue {
            texttrackcue: TextTrackCue::new_inherited(
                DOMString::default(),
                start_time,
                end_time,
                None,
            ),
            region: DomRefCell::new(None),
            vertical: Cell::new(DirectionSetting::default()),
            snap_to_lines: Cell::new(true),
            line: DomRefCell::new(LineAndPositionSetting::Auto),
            line_align: Cell::new(LineAlignSetting::Start),
            position: DomRefCell::new(LineAndPositionSetting::Auto),
            position_align: Cell::new(PositionAlignSetting::Auto),
            size: Cell::new(100_f64),
            align: Cell::new(AlignSetting::Center),
            text: DomRefCell::new(text),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        start_time: f64,
        end_time: f64,
        text: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(start_time, end_time, text)),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        start_time: Finite<f64>,
        end_time: Finite<f64>,
        text: DOMString,
    ) -> DomRoot<Self> {
        VTTCue::new(&window.global(), proto, *start_time, *end_time, text)
    }
}

impl VTTCueMethods for VTTCue {
    // https://w3c.github.io/webvtt/#dom-vttcue-region
    fn GetRegion(&self) -> Option<DomRoot<VTTRegion>> {
        self.region
            .borrow()
            .as_ref()
            .map(|r| DomRoot::from_ref(&**r))
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-region
    fn SetRegion(&self, value: Option<&VTTRegion>) {
        *self.region.borrow_mut() = value.map(Dom::from_ref)
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-vertical
    fn Vertical(&self) -> DirectionSetting {
        self.vertical.get()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-vertical
    fn SetVertical(&self, value: DirectionSetting) {
        self.vertical.set(value);
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-snaptolines
    fn SnapToLines(&self) -> bool {
        self.snap_to_lines.get()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-snaptolines
    fn SetSnapToLines(&self, value: bool) {
        self.snap_to_lines.set(value)
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-line
    fn Line(&self) -> VTTCueBinding::LineAndPositionSetting {
        VTTCueBinding::LineAndPositionSetting::from(self.line.borrow().clone())
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-line
    fn SetLine(&self, value: VTTCueBinding::LineAndPositionSetting) {
        *self.line.borrow_mut() = value.into();
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-linealign
    fn LineAlign(&self) -> LineAlignSetting {
        self.line_align.get()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-linealign
    fn SetLineAlign(&self, value: LineAlignSetting) {
        self.line_align.set(value);
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-position
    fn Position(&self) -> VTTCueBinding::LineAndPositionSetting {
        VTTCueBinding::LineAndPositionSetting::from(self.position.borrow().clone())
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-position
    fn SetPosition(&self, value: VTTCueBinding::LineAndPositionSetting) -> ErrorResult {
        if let VTTCueBinding::LineAndPositionSetting::Double(x) = value {
            if *x < 0_f64 || *x > 100_f64 {
                return Err(Error::IndexSize);
            }
        }

        *self.position.borrow_mut() = value.into();
        Ok(())
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-positionalign
    fn PositionAlign(&self) -> PositionAlignSetting {
        self.position_align.get()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-positionalign
    fn SetPositionAlign(&self, value: PositionAlignSetting) {
        self.position_align.set(value);
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-size
    fn Size(&self) -> Finite<f64> {
        Finite::wrap(self.size.get())
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-size
    fn SetSize(&self, value: Finite<f64>) -> ErrorResult {
        if *value < 0_f64 || *value > 100_f64 {
            return Err(Error::IndexSize);
        }

        self.size.set(*value);
        Ok(())
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-align
    fn Align(&self) -> AlignSetting {
        self.align.get()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-align
    fn SetAlign(&self, value: AlignSetting) {
        self.align.set(value);
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-text
    fn Text(&self) -> DOMString {
        self.text.borrow().clone()
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-text
    fn SetText(&self, value: DOMString) {
        *self.text.borrow_mut() = value;
    }

    // https://w3c.github.io/webvtt/#dom-vttcue-getcueashtml
    fn GetCueAsHTML(&self) -> DomRoot<DocumentFragment> {
        todo!()
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
