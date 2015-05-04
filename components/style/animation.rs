/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::ComputedValues;
use properties::longhands::transition_property::computed_value::TransitionProperty;
use properties::longhands::transition_timing_function::computed_value::{StartEnd};
use properties::longhands::transition_timing_function::computed_value::{TransitionTimingFunction};
use properties::longhands::z_index::computed_value::T as ZIndex;
use properties::longhands::visibility::computed_value::T as Visibility;
use properties::longhands::vertical_align::computed_value::T as VerticalAlign;
use properties::longhands::border_spacing::computed_value::T as BorderSpacing;
use properties::longhands::line_height::computed_value::T as LineHeight;
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::clip::computed_value::ClipRect;
use properties::longhands::text_shadow::computed_value::TextShadow;
use properties::longhands::text_shadow::computed_value::T as TextShadowList;
use properties::longhands::background_position::computed_value::T as BackgroundPosition;
use properties::longhands::transition_property;
use values::computed::{LengthOrPercentageOrAuto, LengthOrPercentageOrNone, LengthOrPercentage, Length, Time};
use values::CSSFloat;
use cssparser::{RGBA, Color};

use std::cmp::Ordering;
use std::iter::repeat;
use util::bezier::Bezier;
use util::geometry::Au;

#[derive(Clone, Debug)]
pub struct PropertyAnimation {
    property: AnimatedProperty,
    timing_function: TransitionTimingFunction,
    duration: Time,
}

impl PropertyAnimation {
    /// Creates a new property animation for the given transition index and old and new styles.
    /// Any number of animations may be returned, from zero (if the property did not animate) to
    /// one (for a single transition property) to arbitrarily many (for `all`).
    pub fn from_transition(transition_index: usize,
                           old_style: &ComputedValues,
                           new_style: &mut ComputedValues)
                           -> Vec<PropertyAnimation> {
        let mut result = Vec::new();
        let transition_property =
            new_style.get_animation().transition_property.0[transition_index];
        if transition_property != TransitionProperty::All {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(transition_property,
                                                                transition_index,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
            return result
        }

        for transition_property in
                transition_property::computed_value::ALL_TRANSITION_PROPERTIES.iter() {
            if let Some(property_animation) =
                    PropertyAnimation::from_transition_property(*transition_property,
                                                                transition_index,
                                                                old_style,
                                                                new_style) {
                result.push(property_animation)
            }
        }

        result
    }

    fn from_transition_property(transition_property: TransitionProperty,
                                transition_index: usize,
                                old_style: &ComputedValues,
                                new_style: &mut ComputedValues)
                                -> Option<PropertyAnimation> {
        let animation_style = new_style.get_animation();
        macro_rules! match_transition {
                ( $( [$name:ident; $structname:ident; $field:ident] ),* ) => {
                    match transition_property {
                        TransitionProperty::All => {
                            panic!("Don't use `TransitionProperty::All` with \
                                   `PropertyAnimation::from_transition_property`!")
                        }
                        $(
                            TransitionProperty::$name => {
                                AnimatedProperty::$name(old_style.$structname().$field,
                                                        new_style.$structname().$field)
                            }
                        )*
                        TransitionProperty::TextShadow => {
                            AnimatedProperty::TextShadow(old_style.get_effects().text_shadow.clone(),
                                                         new_style.get_effects().text_shadow.clone())
                        }
                    }
                }
        }
        let animated_property = match_transition!(
            [BackgroundColor; get_background; background_color],
            [BackgroundPosition; get_background; background_position],
            [BorderBottomColor; get_border; border_bottom_color],
            [BorderBottomWidth; get_border; border_bottom_width],
            [BorderLeftColor; get_border; border_left_color],
            [BorderLeftWidth; get_border; border_left_width],
            [BorderRightColor; get_border; border_right_color],
            [BorderRightWidth; get_border; border_right_width],
            [BorderSpacing; get_inheritedtable; border_spacing],
            [BorderTopColor; get_border; border_top_color],
            [BorderTopWidth; get_border; border_top_width],
            [Bottom; get_positionoffsets; bottom],
            [Color; get_color; color],
            [Clip; get_effects; clip],
            [FontSize; get_font; font_size],
            [FontWeight; get_font; font_weight],
            [Height; get_box; height],
            [Left; get_positionoffsets; bottom],
            [LetterSpacing; get_inheritedtext; letter_spacing],
            [LineHeight; get_inheritedbox; line_height],
            [MarginBottom; get_margin; margin_bottom],
            [MarginLeft; get_margin; margin_left],
            [MarginRight; get_margin; margin_right],
            [MarginTop; get_margin; margin_top],
            [MaxHeight; get_box; max_height],
            [MaxWidth; get_box; max_width],
            [MinHeight; get_box; min_height],
            [MinWidth; get_box; min_width],
            [Opacity; get_effects; opacity],
            [OutlineColor; get_outline; outline_color],
            [OutlineWidth; get_outline; outline_width],
            [PaddingBottom; get_margin; margin_bottom],
            [PaddingLeft; get_margin; margin_left],
            [PaddingRight; get_margin; margin_right],
            [PaddingTop; get_margin; margin_top],
            [Right; get_positionoffsets; right],
            [TextIndent; get_inheritedtext; text_indent],
            [Top; get_positionoffsets; top],
            [VerticalAlign; get_box; vertical_align],
            [Visibility; get_inheritedbox; visibility],
            [Width; get_box; width],
            [WordSpacing; get_inheritedtext; word_spacing],
            [ZIndex; get_box; z_index]);

        let property_animation = PropertyAnimation {
            property: animated_property,
            timing_function:
                *animation_style.transition_timing_function.0.get_mod(transition_index),
            duration: *animation_style.transition_duration.0.get_mod(transition_index),
        };
        if property_animation.does_not_animate() {
            None
        } else {
            Some(property_animation)
        }
    }

    pub fn update(&self, style: &mut ComputedValues, time: f32) {
        let progress = match self.timing_function {
            TransitionTimingFunction::CubicBezier(p1, p2) => {
                // See `WebCore::AnimationBase::solveEpsilon(double)` in WebKit.
                let epsilon = 1.0 / (200.0 * self.duration.seconds());
                Bezier::new(p1, p2).solve(time, epsilon)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::Start) => {
                (time * (steps as f32)).ceil() / (steps as f32)
            }
            TransitionTimingFunction::Steps(steps, StartEnd::End) => {
                (time * (steps as f32)).floor() / (steps as f32)
            }
        };

        macro_rules! match_property(
            ( $( [$name:ident; $structname:ident; $field:ident] ),* ) => {
                match self.property {
                    $(
                        AnimatedProperty::$name(ref start, ref end) => {
                            if let Some(value) = start.interpolate(end, progress) {
                                style.$structname().$field = value
                            }
                        }
                    )*
                }
            });
        match_property!(
            [BackgroundColor; mutate_background; background_color],
            [BackgroundPosition; mutate_background; background_position],
            [BorderBottomColor; mutate_border; border_bottom_color],
            [BorderBottomWidth; mutate_border; border_bottom_width],
            [BorderLeftColor; mutate_border; border_left_color],
            [BorderLeftWidth; mutate_border; border_left_width],
            [BorderRightColor; mutate_border; border_right_color],
            [BorderRightWidth; mutate_border; border_right_width],
            [BorderSpacing; mutate_inheritedtable; border_spacing],
            [BorderTopColor; mutate_border; border_top_color],
            [BorderTopWidth; mutate_border; border_top_width],
            [Bottom; mutate_positionoffsets; bottom],
            [Color; mutate_color; color],
            [Clip; mutate_effects; clip],
            [FontSize; mutate_font; font_size],
            [FontWeight; mutate_font; font_weight],
            [Height; mutate_box; height],
            [Left; mutate_positionoffsets; bottom],
            [LetterSpacing; mutate_inheritedtext; letter_spacing],
            [LineHeight; mutate_inheritedbox; line_height],
            [MarginBottom; mutate_margin; margin_bottom],
            [MarginLeft; mutate_margin; margin_left],
            [MarginRight; mutate_margin; margin_right],
            [MarginTop; mutate_margin; margin_top],
            [MaxHeight; mutate_box; max_height],
            [MaxWidth; mutate_box; max_width],
            [MinHeight; mutate_box; min_height],
            [MinWidth; mutate_box; min_width],
            [Opacity; mutate_effects; opacity],
            [OutlineColor; mutate_outline; outline_color],
            [OutlineWidth; mutate_outline; outline_width],
            [PaddingBottom; mutate_margin; margin_bottom],
            [PaddingLeft; mutate_margin; margin_left],
            [PaddingRight; mutate_margin; margin_right],
            [PaddingTop; mutate_margin; margin_top],
            [Right; mutate_positionoffsets; right],
            [TextIndent; mutate_inheritedtext; text_indent],
            [TextShadow; mutate_effects; text_shadow],
            [Top; mutate_positionoffsets; top],
            [VerticalAlign; mutate_box; vertical_align],
            [Visibility; mutate_inheritedbox; visibility],
            [Width; mutate_box; width],
            [WordSpacing; mutate_inheritedtext; word_spacing],
            [ZIndex; mutate_box; z_index]);
    }

    #[inline]
    fn does_not_animate(&self) -> bool {
        self.property.does_not_animate() || self.duration == Time(0.0)
    }
}

#[derive(Clone, Debug)]
enum AnimatedProperty {
    BackgroundColor(Color, Color),
    BackgroundPosition(BackgroundPosition, BackgroundPosition),
    BorderBottomColor(Color, Color),
    BorderBottomWidth(Length, Length),
    BorderLeftColor(Color, Color),
    BorderLeftWidth(Length, Length),
    BorderRightColor(Color, Color),
    BorderRightWidth(Length, Length),
    BorderSpacing(BorderSpacing, BorderSpacing),
    BorderTopColor(Color, Color),
    BorderTopWidth(Length, Length),
    Bottom(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Color(RGBA, RGBA),
    Clip(Option<ClipRect>, Option<ClipRect>),
    FontSize(Length, Length),
    FontWeight(FontWeight, FontWeight),
    Height(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Left(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    LetterSpacing(Option<Au>, Option<Au>),
    LineHeight(LineHeight, LineHeight),
    MarginBottom(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    MarginLeft(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    MarginRight(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    MarginTop(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    MaxHeight(LengthOrPercentageOrNone, LengthOrPercentageOrNone),
    MaxWidth(LengthOrPercentageOrNone, LengthOrPercentageOrNone),
    MinHeight(LengthOrPercentage, LengthOrPercentage),
    MinWidth(LengthOrPercentage, LengthOrPercentage),
    Opacity(CSSFloat, CSSFloat),
    OutlineColor(Color, Color),
    OutlineWidth(Length, Length),
    PaddingBottom(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    PaddingLeft(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    PaddingRight(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    PaddingTop(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    Right(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    TextIndent(LengthOrPercentage, LengthOrPercentage),
    TextShadow(TextShadowList, TextShadowList),
    Top(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    VerticalAlign(VerticalAlign, VerticalAlign),
    Visibility(Visibility, Visibility),
    Width(LengthOrPercentageOrAuto, LengthOrPercentageOrAuto),
    WordSpacing(Option<Au>, Option<Au>),
    ZIndex(ZIndex, ZIndex),
}

impl AnimatedProperty {
    #[inline]
    fn does_not_animate(&self) -> bool {
        match *self {
            AnimatedProperty::Top(ref a, ref b) |
            AnimatedProperty::Right(ref a, ref b) |
            AnimatedProperty::Bottom(ref a, ref b) |
            AnimatedProperty::Left(ref a, ref b) |
            AnimatedProperty::MarginTop(ref a, ref b) |
            AnimatedProperty::MarginRight(ref a, ref b) |
            AnimatedProperty::MarginBottom(ref a, ref b) |
            AnimatedProperty::MarginLeft(ref a, ref b) |
            AnimatedProperty::PaddingTop(ref a, ref b) |
            AnimatedProperty::PaddingRight(ref a, ref b) |
            AnimatedProperty::PaddingBottom(ref a, ref b) |
            AnimatedProperty::PaddingLeft(ref a, ref b) |
            AnimatedProperty::Width(ref a, ref b) |
            AnimatedProperty::Height(ref a, ref b) => a == b,
            AnimatedProperty::MaxWidth(ref a, ref b) |
            AnimatedProperty::MaxHeight(ref a, ref b) => a == b,
            AnimatedProperty::MinWidth(ref a, ref b) |
            AnimatedProperty::MinHeight(ref a, ref b) |
            AnimatedProperty::TextIndent(ref a, ref b) => a == b,
            AnimatedProperty::FontSize(ref a, ref b) |
            AnimatedProperty::BorderTopWidth(ref a, ref b) |
            AnimatedProperty::BorderRightWidth(ref a, ref b) |
            AnimatedProperty::BorderBottomWidth(ref a, ref b) |
            AnimatedProperty::BorderLeftWidth(ref a, ref b) => a == b,
            AnimatedProperty::BorderTopColor(ref a, ref b) |
            AnimatedProperty::BorderRightColor(ref a, ref b) |
            AnimatedProperty::BorderBottomColor(ref a, ref b) |
            AnimatedProperty::BorderLeftColor(ref a, ref b) |
            AnimatedProperty::OutlineColor(ref a, ref b) |
            AnimatedProperty::BackgroundColor(ref a, ref b) => a == b,
            AnimatedProperty::LineHeight(ref a, ref b) => a == b,
            AnimatedProperty::LetterSpacing(ref a, ref b) => a == b,
            AnimatedProperty::BackgroundPosition(ref a, ref b) => a == b,
            AnimatedProperty::BorderSpacing(ref a, ref b) => a == b,
            AnimatedProperty::Clip(ref a, ref b) => a == b,
            AnimatedProperty::Color(ref a, ref b) => a == b,
            AnimatedProperty::FontWeight(ref a, ref b) => a == b,
            AnimatedProperty::Opacity(ref a, ref b) => a == b,
            AnimatedProperty::OutlineWidth(ref a, ref b) => a == b,
            AnimatedProperty::TextShadow(ref a, ref b) => a == b,
            AnimatedProperty::VerticalAlign(ref a, ref b) => a == b,
            AnimatedProperty::Visibility(ref a, ref b) => a == b,
            AnimatedProperty::WordSpacing(ref a, ref b) => a == b,
            AnimatedProperty::ZIndex(ref a, ref b) => a == b,
        }
    }
}

trait Interpolate {
    fn interpolate(&self, other: &Self, time: f32) -> Option<Self>;
}

impl Interpolate for Au {
    #[inline]
    fn interpolate(&self, other: &Au, time: f32) -> Option<Au> {
        Some(Au((self.0 as f32 + (other.0 as f32 - self.0 as f32) * time).round() as i32))
    }
}

impl <T> Interpolate for Option<T> where T:Interpolate {
    #[inline]
    fn interpolate(&self, other: &Option<T>, time: f32) -> Option<Option<T>> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(Some(value))
                })
            }
            (_, _) => None
        }
    }
}

impl Interpolate for f32 {
    #[inline]
    fn interpolate(&self, other: &f32, time: f32) -> Option<f32> {
        Some(*self + (*other - *self) * time)
    }
}

impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, other: &f64, time: f32) -> Option<f64> {
        Some(*self + (*other - *self) * (time as f64))
    }
}

impl Interpolate for i32 {
    #[inline]
    fn interpolate(&self, other: &i32, time: f32) -> Option<i32> {
        let a = *self as f32;
        let b = *other as f32;
        Some((a + (b - a) * time).round() as i32)
    }
}

impl Interpolate for Visibility {
    #[inline]
    fn interpolate(&self, other: &Visibility, time: f32)
                   -> Option<Visibility> {
        match (*self, *other) {
            (Visibility::visible, _) | (_, Visibility::visible) => {
                if time >= 0.0 && time <= 1.0 {
                    Some(Visibility::visible)
                } else if time < 0.0 {
                    Some(*self)
                } else {
                    Some(*other)
                }
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for ZIndex {
    #[inline]
    fn interpolate(&self, other: &ZIndex, time: f32)
                   -> Option<ZIndex> {
        match (*self, *other) {
            (ZIndex::Number(ref this),
             ZIndex::Number(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(ZIndex::Number(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for VerticalAlign {
    #[inline]
    fn interpolate(&self, other: &VerticalAlign, time: f32)
                   -> Option<VerticalAlign> {
        match (*self, *other) {
            (VerticalAlign::Length(ref this),
             VerticalAlign::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(VerticalAlign::Length(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for BorderSpacing {
    #[inline]
    fn interpolate(&self, other: &BorderSpacing, time: f32)
                   -> Option<BorderSpacing> {
        self.horizontal.interpolate(&other.horizontal, time).and_then(|horizontal| {
            self.vertical.interpolate(&other.vertical, time).and_then(|vertical| {
                Some(BorderSpacing { horizontal: horizontal, vertical: vertical })
            })
        })
    }
}

impl Interpolate for RGBA {
    #[inline]
    fn interpolate(&self, other: &RGBA, time: f32) -> Option<RGBA> {
        match (self.red.interpolate(&other.red, time),
               self.green.interpolate(&other.green, time),
               self.blue.interpolate(&other.blue, time),
               self.alpha.interpolate(&other.alpha, time)) {
            (Some(red), Some(green), Some(blue), Some(alpha)) => {
                Some(RGBA { red: red, green: green, blue: blue, alpha: alpha })
            }
            (_, _, _, _) => None
        }
    }
}

impl Interpolate for Color {
    #[inline]
    fn interpolate(&self, other: &Color, time: f32) -> Option<Color> {
        match (*self, *other) {
            (Color::RGBA(ref this), Color::RGBA(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(Color::RGBA(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for LengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentage, time: f32)
                   -> Option<LengthOrPercentage> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentage::Length(value))
                })
            }
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentage::Percentage(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for LengthOrPercentageOrAuto {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentageOrAuto, time: f32)
                   -> Option<LengthOrPercentageOrAuto> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Length(value))
                })
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Percentage(value))
                })
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Some(LengthOrPercentageOrAuto::Auto)
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for LengthOrPercentageOrNone {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentageOrNone, time: f32)
                   -> Option<LengthOrPercentageOrNone> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrNone::Length(value))
                })
            }
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrNone::Percentage(value))
                })
            }
            (LengthOrPercentageOrNone::None, LengthOrPercentageOrNone::None) => {
                Some(LengthOrPercentageOrNone::None)
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for LineHeight {
    #[inline]
    fn interpolate(&self, other: &LineHeight, time: f32)
                   -> Option<LineHeight> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LineHeight::Length(value))
                })
            }
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LineHeight::Number(value))
                })
            }
            (LineHeight::Normal, LineHeight::Normal) => {
                Some(LineHeight::Normal)
            }
            (_, _) => None,
        }
    }
}

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Interpolate for FontWeight {
    #[inline]
    fn interpolate(&self, other: &FontWeight, time: f32)
                   -> Option<FontWeight> {
        let a = (*self as u32) as f32;
        let b = (*other as u32) as f32;
        let weight = a + (b - a) * time;
        Some(if weight < 150. {
            FontWeight::Weight100
        } else if weight < 250. {
            FontWeight::Weight200
        } else if weight < 350. {
            FontWeight::Weight300
        } else if weight < 450. {
            FontWeight::Weight400
        } else if weight < 550. {
            FontWeight::Weight500
        } else if weight < 650. {
            FontWeight::Weight600
        } else if weight < 750. {
            FontWeight::Weight700
        } else if weight < 850. {
            FontWeight::Weight800
        } else {
            FontWeight::Weight900
        })
    }
}

impl Interpolate for ClipRect {
    #[inline]
    fn interpolate(&self, other: &ClipRect, time: f32)
                   -> Option<ClipRect> {
        match (self.top.interpolate(&other.top, time),
               self.right.interpolate(&other.right, time),
               self.bottom.interpolate(&other.bottom, time),
               self.left.interpolate(&other.left, time)) {
            (Some(top), Some(right), Some(bottom), Some(left)) => {
                Some(ClipRect { top: top, right: right, bottom: bottom, left: left })
            },
            (_, _, _, _) => None,
        }
    }
}

impl Interpolate for BackgroundPosition {
    #[inline]
    fn interpolate(&self, other: &BackgroundPosition, time: f32)
                   -> Option<BackgroundPosition> {
        match (self.horizontal.interpolate(&other.horizontal, time),
               self.vertical.interpolate(&other.vertical, time)) {
            (Some(horizontal), Some(vertical)) => {
                Some(BackgroundPosition { horizontal: horizontal, vertical: vertical })
            },
            (_, _) => None,
        }
    }
}

impl Interpolate for TextShadow {
    #[inline]
    fn interpolate(&self, other: &TextShadow, time: f32)
                   -> Option<TextShadow> {
        match (self.offset_x.interpolate(&other.offset_x, time),
               self.offset_y.interpolate(&other.offset_y, time),
               self.blur_radius.interpolate(&other.blur_radius, time),
               self.color.interpolate(&other.color, time)) {
            (Some(offset_x), Some(offset_y), Some(blur_radius), Some(color)) => {
                Some(TextShadow { offset_x: offset_x, offset_y: offset_y, blur_radius: blur_radius, color: color })
            },
            (_, _, _, _) => None,
        }
    }
}

impl Interpolate for TextShadowList {
    #[inline]
    fn interpolate(&self, other: &TextShadowList, time: f32)
                   -> Option<TextShadowList> {
        let zero = TextShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            blur_radius: Au(0),
            color: Color::RGBA(RGBA {
                red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0
            })
        };

        let interpolate_each = |(a, b): (&TextShadow, &TextShadow)| {
            a.interpolate(b, time).unwrap()
        };

        Some(TextShadowList(match self.0.len().cmp(&other.0.len()) {
            Ordering::Less => other.0.iter().chain(repeat(&zero)).zip(other.0.iter()).map(interpolate_each).collect(),
            _ => self.0.iter().zip(other.0.iter().chain(repeat(&zero))).map(interpolate_each).collect(),
        }))
    }
}

/// Accesses an element of an array, "wrapping around" using modular arithmetic. This is needed
/// to handle values of differing lengths according to CSS-TRANSITIONS § 2.
pub trait GetMod {
    type Item;
    fn get_mod(&self, i: usize) -> &Self::Item;
}

impl<T> GetMod for Vec<T> {
    type Item = T;
    fn get_mod(&self, i: usize) -> &T {
        &(*self)[i % self.len()]
    }
}

