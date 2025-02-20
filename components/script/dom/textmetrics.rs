/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TextMetricsBinding::TextMetricsMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct TextMetrics {
    reflector_: Reflector,
    width: Finite<f64>,
    actualBoundingBoxLeft: Finite<f64>,
    actualBoundingBoxRight: Finite<f64>,
    fontBoundingBoxAscent: Finite<f64>,
    fontBoundingBoxDescent: Finite<f64>,
    actualBoundingBoxAscent: Finite<f64>,
    actualBoundingBoxDescent: Finite<f64>,
    emHeightAscent: Finite<f64>,
    emHeightDescent: Finite<f64>,
    hangingBaseline: Finite<f64>,
    alphabeticBaseline: Finite<f64>,
    ideographicBaseline: Finite<f64>,
}

#[allow(non_snake_case)]
impl TextMetrics {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        width: f64,
        actualBoundingBoxLeft: f64,
        actualBoundingBoxRight: f64,
        fontBoundingBoxAscent: f64,
        fontBoundingBoxDescent: f64,
        actualBoundingBoxAscent: f64,
        actualBoundingBoxDescent: f64,
        emHeightAscent: f64,
        emHeightDescent: f64,
        hangingBaseline: f64,
        alphabeticBaseline: f64,
        ideographicBaseline: f64,
    ) -> TextMetrics {
        TextMetrics {
            reflector_: Reflector::new(),
            width: Finite::wrap(width),
            actualBoundingBoxLeft: Finite::wrap(actualBoundingBoxLeft),
            actualBoundingBoxRight: Finite::wrap(actualBoundingBoxRight),
            fontBoundingBoxAscent: Finite::wrap(fontBoundingBoxAscent),
            fontBoundingBoxDescent: Finite::wrap(fontBoundingBoxDescent),
            actualBoundingBoxAscent: Finite::wrap(actualBoundingBoxAscent),
            actualBoundingBoxDescent: Finite::wrap(actualBoundingBoxDescent),
            emHeightAscent: Finite::wrap(emHeightAscent),
            emHeightDescent: Finite::wrap(emHeightDescent),
            hangingBaseline: Finite::wrap(hangingBaseline),
            alphabeticBaseline: Finite::wrap(alphabeticBaseline),
            ideographicBaseline: Finite::wrap(ideographicBaseline),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        width: f64,
        actualBoundingBoxLeft: f64,
        actualBoundingBoxRight: f64,
        fontBoundingBoxAscent: f64,
        fontBoundingBoxDescent: f64,
        actualBoundingBoxAscent: f64,
        actualBoundingBoxDescent: f64,
        emHeightAscent: f64,
        emHeightDescent: f64,
        hangingBaseline: f64,
        alphabeticBaseline: f64,
        ideographicBaseline: f64,
        can_gc: CanGc,
    ) -> DomRoot<TextMetrics> {
        reflect_dom_object(
            Box::new(TextMetrics::new_inherited(
                width,
                actualBoundingBoxLeft,
                actualBoundingBoxRight,
                fontBoundingBoxAscent,
                fontBoundingBoxDescent,
                actualBoundingBoxAscent,
                actualBoundingBoxDescent,
                emHeightAscent,
                emHeightDescent,
                hangingBaseline,
                alphabeticBaseline,
                ideographicBaseline,
            )),
            global,
            can_gc,
        )
    }
}

impl TextMetricsMethods<crate::DomTypeHolder> for TextMetrics {
    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-width>
    fn Width(&self) -> Finite<f64> {
        self.width
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-actualboundingboxleft>
    fn ActualBoundingBoxLeft(&self) -> Finite<f64> {
        self.actualBoundingBoxLeft
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-actualboundingboxright>
    fn ActualBoundingBoxRight(&self) -> Finite<f64> {
        self.actualBoundingBoxRight
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-fontboundingboxascent>
    fn FontBoundingBoxAscent(&self) -> Finite<f64> {
        self.fontBoundingBoxAscent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-fontboundingboxascent>
    fn FontBoundingBoxDescent(&self) -> Finite<f64> {
        self.fontBoundingBoxDescent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-actualboundingboxascent>
    fn ActualBoundingBoxAscent(&self) -> Finite<f64> {
        self.actualBoundingBoxAscent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-actualboundingboxdescent>
    fn ActualBoundingBoxDescent(&self) -> Finite<f64> {
        self.actualBoundingBoxDescent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-emheightascent>
    fn EmHeightAscent(&self) -> Finite<f64> {
        self.emHeightAscent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-emheightdescent>
    fn EmHeightDescent(&self) -> Finite<f64> {
        self.emHeightDescent
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-hangingbaseline>
    fn HangingBaseline(&self) -> Finite<f64> {
        self.hangingBaseline
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-alphabeticbaseline>
    fn AlphabeticBaseline(&self) -> Finite<f64> {
        self.alphabeticBaseline
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textmetrics-ideographicbaseline>
    fn IdeographicBaseline(&self) -> Finite<f64> {
        self.ideographicBaseline
    }
}
