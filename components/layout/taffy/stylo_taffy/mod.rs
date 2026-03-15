/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Conversion functions from Stylo types to Taffy types

mod convert;
mod wrapper;

pub(crate) use wrapper::TaffyStyloStyle;

/// Private module of type aliases so we can refer to stylo types with nicer names
mod stylo {
    pub(crate) use style::properties::generated::longhands::box_sizing::computed_value::T as BoxSizing;
    pub(crate) use style::properties::generated::longhands::direction::computed_value::T as Direction;
    pub(crate) use style::properties::longhands::aspect_ratio::computed_value::T as AspectRatio;
    pub(crate) use style::properties::longhands::position::computed_value::T as Position;
    pub(crate) use style::values::computed::length_percentage::{
        CalcLengthPercentage, Unpacked as UnpackedLengthPercentage,
    };
    pub(crate) use style::values::computed::{LengthPercentage, Percentage};
    pub(crate) use style::values::generics::NonNegative;
    pub(crate) use style::values::generics::length::{
        GenericLengthPercentageOrNormal, GenericMargin, GenericMaxSize, GenericSize,
    };
    pub(crate) use style::values::generics::position::{Inset as GenericInset, PreferredRatio};
    pub(crate) use style::values::specified::align::{AlignFlags, ContentDistribution};
    pub(crate) use style::values::specified::box_::{
        Display, DisplayInside, DisplayOutside, Overflow,
    };
    pub(crate) type MarginVal = GenericMargin<LengthPercentage>;
    pub(crate) type InsetVal = GenericInset<Percentage, LengthPercentage>;
    pub(crate) type Size = GenericSize<NonNegative<LengthPercentage>>;
    pub(crate) type MaxSize = GenericMaxSize<NonNegative<LengthPercentage>>;

    pub(crate) type Gap = GenericLengthPercentageOrNormal<NonNegative<LengthPercentage>>;

    pub(crate) use style::computed_values::grid_auto_flow::T as GridAutoFlow;
    pub(crate) use style::values::computed::GridLine;
    pub(crate) use style::values::generics::grid::{
        RepeatCount, TrackBreadth, TrackListValue, TrackSize,
    };
    pub(crate) use style::values::specified::GenericGridTemplateComponent;
}
