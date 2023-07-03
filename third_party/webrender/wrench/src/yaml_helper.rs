/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{Angle, Size2D};
use crate::parse_function::parse_function;
use std::f32;
use std::str::FromStr;
use webrender::api::*;
use webrender::api::units::*;
use yaml_rust::{Yaml, YamlLoader};

pub trait YamlHelper {
    fn as_f32(&self) -> Option<f32>;
    fn as_force_f32(&self) -> Option<f32>;
    fn as_vec_f32(&self) -> Option<Vec<f32>>;
    fn as_vec_u32(&self) -> Option<Vec<u32>>;
    fn as_vec_u64(&self) -> Option<Vec<u64>>;
    fn as_pipeline_id(&self) -> Option<PipelineId>;
    fn as_rect(&self) -> Option<LayoutRect>;
    fn as_size(&self) -> Option<LayoutSize>;
    fn as_point(&self) -> Option<LayoutPoint>;
    fn as_vector(&self) -> Option<LayoutVector2D>;
    fn as_matrix4d(&self) -> Option<LayoutTransform>;
    fn as_transform(&self, transform_origin: &LayoutPoint) -> Option<LayoutTransform>;
    fn as_colorf(&self) -> Option<ColorF>;
    fn as_vec_colorf(&self) -> Option<Vec<ColorF>>;
    fn as_px_to_f32(&self) -> Option<f32>;
    fn as_pt_to_f32(&self) -> Option<f32>;
    fn as_vec_string(&self) -> Option<Vec<String>>;
    fn as_border_radius_component(&self) -> LayoutSize;
    fn as_border_radius(&self) -> Option<BorderRadius>;
    fn as_transform_style(&self) -> Option<TransformStyle>;
    fn as_raster_space(&self) -> Option<RasterSpace>;
    fn as_clip_mode(&self) -> Option<ClipMode>;
    fn as_mix_blend_mode(&self) -> Option<MixBlendMode>;
    fn as_filter_op(&self) -> Option<FilterOp>;
    fn as_vec_filter_op(&self) -> Option<Vec<FilterOp>>;
    fn as_filter_data(&self) -> Option<FilterData>;
    fn as_vec_filter_data(&self) -> Option<Vec<FilterData>>;
    fn as_filter_input(&self) -> Option<FilterPrimitiveInput>;
    fn as_filter_primitive(&self) -> Option<FilterPrimitive>;
    fn as_vec_filter_primitive(&self) -> Option<Vec<FilterPrimitive>>;
    fn as_color_space(&self) -> Option<ColorSpace>;
}

fn string_to_color(color: &str) -> Option<ColorF> {
    match color {
        "red" => Some(ColorF::new(1.0, 0.0, 0.0, 1.0)),
        "green" => Some(ColorF::new(0.0, 1.0, 0.0, 1.0)),
        "blue" => Some(ColorF::new(0.0, 0.0, 1.0, 1.0)),
        "white" => Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
        "black" => Some(ColorF::new(0.0, 0.0, 0.0, 1.0)),
        "yellow" => Some(ColorF::new(1.0, 1.0, 0.0, 1.0)),
        "transparent" => Some(ColorF::new(1.0, 1.0, 1.0, 0.0)),
        s => {
            let items: Vec<f32> = s.split_whitespace()
                .map(|s| f32::from_str(s).unwrap())
                .collect();
            if items.len() == 3 {
                Some(ColorF::new(
                    items[0] / 255.0,
                    items[1] / 255.0,
                    items[2] / 255.0,
                    1.0,
                ))
            } else if items.len() == 4 {
                Some(ColorF::new(
                    items[0] / 255.0,
                    items[1] / 255.0,
                    items[2] / 255.0,
                    items[3],
                ))
            } else {
                None
            }
        }
    }
}

pub trait StringEnum: Sized {
    fn from_str(_: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;
}

macro_rules! define_string_enum {
    ($T:ident, [ $( $y:ident = $x:expr ),* ]) => {
        impl StringEnum for $T {
            fn from_str(text: &str) -> Option<$T> {
                match text {
                $( $x => Some($T::$y), )*
                    _ => {
                        println!("Unrecognized {} value '{}'", stringify!($T), text);
                        None
                    }
                }
            }
            fn as_str(&self) -> &'static str {
                match *self {
                $( $T::$y => $x, )*
                }
            }
        }
    }
}

define_string_enum!(TransformStyle, [Flat = "flat", Preserve3D = "preserve-3d"]);

define_string_enum!(
    MixBlendMode,
    [
        Normal = "normal",
        Multiply = "multiply",
        Screen = "screen",
        Overlay = "overlay",
        Darken = "darken",
        Lighten = "lighten",
        ColorDodge = "color-dodge",
        ColorBurn = "color-burn",
        HardLight = "hard-light",
        SoftLight = "soft-light",
        Difference = "difference",
        Exclusion = "exclusion",
        Hue = "hue",
        Saturation = "saturation",
        Color = "color",
        Luminosity = "luminosity"
    ]
);

define_string_enum!(
    LineOrientation,
    [Horizontal = "horizontal", Vertical = "vertical"]
);

define_string_enum!(
    LineStyle,
    [
        Solid = "solid",
        Dotted = "dotted",
        Dashed = "dashed",
        Wavy = "wavy"
    ]
);

define_string_enum!(ClipMode, [Clip = "clip", ClipOut = "clip-out"]);

define_string_enum!(
    ComponentTransferFuncType,
    [
        Identity = "Identity",
        Table = "Table",
        Discrete = "Discrete",
        Linear = "Linear",
        Gamma = "Gamma"
    ]
);

define_string_enum!(
    ColorSpace,
    [
        Srgb = "srgb",
        LinearRgb = "linear-rgb"
    ]
);

// Rotate around `axis` by `degrees` angle
fn make_rotation(
    origin: &LayoutPoint,
    degrees: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) -> LayoutTransform {
    let pre_transform = LayoutTransform::translation(-origin.x, -origin.y, -0.0);
    let post_transform = LayoutTransform::translation(origin.x, origin.y, 0.0);

    let theta = 2.0f32 * f32::consts::PI - degrees.to_radians();
    let transform =
        LayoutTransform::identity().pre_rotate(axis_x, axis_y, axis_z, Angle::radians(theta));

    pre_transform.then(&transform).then(&post_transform)
}

pub fn make_perspective(
    origin: LayoutPoint,
    perspective: f32,
) -> LayoutTransform {
    let pre_transform = LayoutTransform::translation(-origin.x, -origin.y, -0.0);
    let post_transform = LayoutTransform::translation(origin.x, origin.y, 0.0);
    let transform = LayoutTransform::perspective(perspective);
    pre_transform.then(&transform).then(&post_transform)
}

// Create a skew matrix, specified in degrees.
fn make_skew(
    skew_x: f32,
    skew_y: f32,
) -> LayoutTransform {
    let alpha = Angle::radians(skew_x.to_radians());
    let beta = Angle::radians(skew_y.to_radians());
    LayoutTransform::skew(alpha, beta)
}

impl YamlHelper for Yaml {
    fn as_f32(&self) -> Option<f32> {
        match *self {
            Yaml::Integer(iv) => Some(iv as f32),
            Yaml::Real(ref sv) => f32::from_str(sv.as_str()).ok(),
            _ => None,
        }
    }

    fn as_force_f32(&self) -> Option<f32> {
        match *self {
            Yaml::Integer(iv) => Some(iv as f32),
            Yaml::String(ref sv) | Yaml::Real(ref sv) => f32::from_str(sv.as_str()).ok(),
            _ => None,
        }
    }

    fn as_vec_f32(&self) -> Option<Vec<f32>> {
        match *self {
            Yaml::String(ref s) | Yaml::Real(ref s) => s.split_whitespace()
                .map(|v| f32::from_str(v))
                .collect::<Result<Vec<_>, _>>()
                .ok(),
            Yaml::Array(ref v) => v.iter()
                .map(|v| match *v {
                    Yaml::Integer(k) => Ok(k as f32),
                    Yaml::String(ref k) | Yaml::Real(ref k) => f32::from_str(k).map_err(|_| false),
                    _ => Err(false),
                })
                .collect::<Result<Vec<_>, _>>()
                .ok(),
            Yaml::Integer(k) => Some(vec![k as f32]),
            _ => None,
        }
    }

    fn as_vec_u32(&self) -> Option<Vec<u32>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|v| v.as_i64().unwrap() as u32).collect())
        } else {
            None
        }
    }

    fn as_vec_u64(&self) -> Option<Vec<u64>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|v| v.as_i64().unwrap() as u64).collect())
        } else {
            None
        }
    }

    fn as_pipeline_id(&self) -> Option<PipelineId> {
        if let Some(v) = self.as_vec() {
            let a = v.get(0).and_then(|v| v.as_i64()).map(|v| v as u32);
            let b = v.get(1).and_then(|v| v.as_i64()).map(|v| v as u32);
            match (a, b) {
                (Some(a), Some(b)) if v.len() == 2 => Some(PipelineId(a, b)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn as_px_to_f32(&self) -> Option<f32> {
        self.as_force_f32()
    }

    fn as_pt_to_f32(&self) -> Option<f32> {
        self.as_force_f32().map(|fv| fv * 16. / 12.)
    }

    fn as_rect(&self) -> Option<LayoutRect> {
        if self.is_badvalue() {
            return None;
        }

        if let Some(nums) = self.as_vec_f32() {
            if nums.len() == 4 {
                return Some(LayoutRect::new(
                    LayoutPoint::new(nums[0], nums[1]),
                    LayoutSize::new(nums[2], nums[3]),
                ));
            }
        }

        None
    }

    fn as_size(&self) -> Option<LayoutSize> {
        if self.is_badvalue() {
            return None;
        }

        if let Some(nums) = self.as_vec_f32() {
            if nums.len() == 2 {
                return Some(LayoutSize::new(nums[0], nums[1]));
            }
        }

        None
    }

    fn as_point(&self) -> Option<LayoutPoint> {
        if self.is_badvalue() {
            return None;
        }

        if let Some(nums) = self.as_vec_f32() {
            if nums.len() == 2 {
                return Some(LayoutPoint::new(nums[0], nums[1]));
            }
        }

        None
    }

    fn as_vector(&self) -> Option<LayoutVector2D> {
        self.as_point().map(|p| p.to_vector())
    }

    fn as_matrix4d(&self) -> Option<LayoutTransform> {
        if let Some(nums) = self.as_vec_f32() {
            assert_eq!(nums.len(), 16, "expected 16 floats, got '{:?}'", self);
            Some(LayoutTransform::new(
                nums[0], nums[1], nums[2], nums[3],
                nums[4], nums[5], nums[6], nums[7],
                nums[8], nums[9], nums[10], nums[11],
                nums[12], nums[13], nums[14], nums[15],
            ))
        } else {
            None
        }
    }

    fn as_transform(&self, transform_origin: &LayoutPoint) -> Option<LayoutTransform> {
        if let Some(transform) = self.as_matrix4d() {
            return Some(transform);
        }

        match *self {
            Yaml::String(ref string) => {
                let mut slice = string.as_str();
                let mut transform = LayoutTransform::identity();
                while !slice.is_empty() {
                    let (function, ref args, reminder) = parse_function(slice);
                    slice = reminder;
                    let mx = match function {
                        "translate" if args.len() >= 2 => {
                            let z = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(0.);
                            LayoutTransform::translation(
                                args[0].parse().unwrap(),
                                args[1].parse().unwrap(),
                                z,
                            )
                        }
                        "rotate" | "rotate-z" if args.len() == 1 => {
                            make_rotation(transform_origin, args[0].parse().unwrap(), 0.0, 0.0, 1.0)
                        }
                        "rotate-x" if args.len() == 1 => {
                            make_rotation(transform_origin, args[0].parse().unwrap(), 1.0, 0.0, 0.0)
                        }
                        "rotate-y" if args.len() == 1 => {
                            make_rotation(transform_origin, args[0].parse().unwrap(), 0.0, 1.0, 0.0)
                        }
                        "scale" if args.len() >= 1 => {
                            let x = args[0].parse().unwrap();
                            // Default to uniform X/Y scale if Y unspecified.
                            let y = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(x);
                            // Default to no Z scale if unspecified.
                            let z = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(1.0);
                            LayoutTransform::scale(x, y, z)
                        }
                        "scale-x" if args.len() == 1 => {
                            LayoutTransform::scale(args[0].parse().unwrap(), 1.0, 1.0)
                        }
                        "scale-y" if args.len() == 1 => {
                            LayoutTransform::scale(1.0, args[0].parse().unwrap(), 1.0)
                        }
                        "scale-z" if args.len() == 1 => {
                            LayoutTransform::scale(1.0, 1.0, args[0].parse().unwrap())
                        }
                        "skew" if args.len() >= 1 => {
                            // Default to no Y skew if unspecified.
                            let skew_y = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.0);
                            make_skew(args[0].parse().unwrap(), skew_y)
                        }
                        "skew-x" if args.len() == 1 => {
                            make_skew(args[0].parse().unwrap(), 0.0)
                        }
                        "skew-y" if args.len() == 1 => {
                            make_skew(0.0, args[0].parse().unwrap())
                        }
                        "perspective" if args.len() == 1 => {
                            LayoutTransform::perspective(args[0].parse().unwrap())
                        }
                        _ => {
                            println!("unknown function {}", function);
                            break;
                        }
                    };
                    transform = transform.then(&mx);
                }
                Some(transform)
            }
            Yaml::Array(ref array) => {
                let transform = array.iter().fold(
                    LayoutTransform::identity(),
                    |u, yaml| match yaml.as_transform(transform_origin) {
                        Some(ref transform) => transform.then(&u),
                        None => u,
                    },
                );
                Some(transform)
            }
            Yaml::BadValue => None,
            _ => {
                println!("unknown transform {:?}", self);
                None
            }
        }
    }

    fn as_colorf(&self) -> Option<ColorF> {
        if let Some(mut nums) = self.as_vec_f32() {
            assert!(
                nums.len() == 3 || nums.len() == 4,
                "color expected a color name, or 3-4 floats; got '{:?}'",
                self
            );

            if nums.len() == 3 {
                nums.push(1.0);
            }
            assert!(nums[3] >= 0.0 && nums[3] <= 1.0,
                    "alpha value should be in the 0-1 range, got {:?}",
                    nums[3]);
            return Some(ColorF::new(
                nums[0] / 255.0,
                nums[1] / 255.0,
                nums[2] / 255.0,
                nums[3],
            ));
        }

        if let Some(s) = self.as_str() {
            string_to_color(s)
        } else {
            None
        }
    }

    fn as_vec_colorf(&self) -> Option<Vec<ColorF>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|v| v.as_colorf().unwrap()).collect())
        } else if let Some(color) = self.as_colorf() {
            Some(vec![color])
        } else {
            None
        }
    }

    fn as_vec_string(&self) -> Option<Vec<String>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|v| v.as_str().unwrap().to_owned()).collect())
        } else if let Some(s) = self.as_str() {
            Some(vec![s.to_owned()])
        } else {
            None
        }
    }

    fn as_border_radius_component(&self) -> LayoutSize {
        if let Yaml::Integer(integer) = *self {
            return LayoutSize::new(integer as f32, integer as f32);
        }
        self.as_size().unwrap_or(Size2D::zero())
    }

    fn as_border_radius(&self) -> Option<BorderRadius> {
        if let Some(size) = self.as_size() {
            return Some(BorderRadius::uniform_size(size));
        }

        match *self {
            Yaml::BadValue => None,
            Yaml::String(ref s) | Yaml::Real(ref s) => {
                let fv = f32::from_str(s).unwrap();
                Some(BorderRadius::uniform(fv))
            }
            Yaml::Integer(v) => Some(BorderRadius::uniform(v as f32)),
            Yaml::Array(ref array) if array.len() == 4 => {
                let top_left = array[0].as_border_radius_component();
                let top_right = array[1].as_border_radius_component();
                let bottom_left = array[2].as_border_radius_component();
                let bottom_right = array[3].as_border_radius_component();
                Some(BorderRadius {
                    top_left,
                    top_right,
                    bottom_left,
                    bottom_right,
                })
            }
            Yaml::Hash(_) => {
                let top_left = self["top-left"].as_border_radius_component();
                let top_right = self["top-right"].as_border_radius_component();
                let bottom_left = self["bottom-left"].as_border_radius_component();
                let bottom_right = self["bottom-right"].as_border_radius_component();
                Some(BorderRadius {
                    top_left,
                    top_right,
                    bottom_left,
                    bottom_right,
                })
            }
            _ => {
                panic!("Invalid border radius specified: {:?}", self);
            }
        }
    }

    fn as_transform_style(&self) -> Option<TransformStyle> {
        self.as_str().and_then(|x| StringEnum::from_str(x))
    }

    fn as_raster_space(&self) -> Option<RasterSpace> {
        self.as_str().and_then(|s| {
            match parse_function(s) {
                ("screen", _, _) => {
                    Some(RasterSpace::Screen)
                }
                ("local", ref args, _) if args.len() == 1 => {
                    Some(RasterSpace::Local(args[0].parse().unwrap()))
                }
                f => {
                    panic!("error parsing raster space {:?}", f);
                }
            }
        })
    }

    fn as_mix_blend_mode(&self) -> Option<MixBlendMode> {
        self.as_str().and_then(|x| StringEnum::from_str(x))
    }

    fn as_clip_mode(&self) -> Option<ClipMode> {
        self.as_str().and_then(|x| StringEnum::from_str(x))
    }

    fn as_filter_op(&self) -> Option<FilterOp> {
        if let Some(s) = self.as_str() {
            match parse_function(s) {
                ("identity", _, _) => {
                    Some(FilterOp::Identity)
                }
                ("component-transfer", _, _) => {
                    Some(FilterOp::ComponentTransfer)
                }
                ("blur", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Blur(args[0].parse().unwrap()))
                }
                ("brightness", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Brightness(args[0].parse().unwrap()))
                }
                ("contrast", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Contrast(args[0].parse().unwrap()))
                }
                ("grayscale", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Grayscale(args[0].parse().unwrap()))
                }
                ("hue-rotate", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::HueRotate(args[0].parse().unwrap()))
                }
                ("invert", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Invert(args[0].parse().unwrap()))
                }
                ("opacity", ref args, _) if args.len() == 1 => {
                    let amount: f32 = args[0].parse().unwrap();
                    Some(FilterOp::Opacity(amount.into(), amount))
                }
                ("saturate", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Saturate(args[0].parse().unwrap()))
                }
                ("sepia", ref args, _) if args.len() == 1 => {
                    Some(FilterOp::Sepia(args[0].parse().unwrap()))
                }
                ("srgb-to-linear", _, _)  => Some(FilterOp::SrgbToLinear),
                ("linear-to-srgb", _, _)  => Some(FilterOp::LinearToSrgb),
                ("drop-shadow", ref args, _) if args.len() == 3 => {
                    let str = format!("---\noffset: {}\nblur-radius: {}\ncolor: {}\n", args[0], args[1], args[2]);
                    let mut yaml_doc = YamlLoader::load_from_str(&str).expect("Failed to parse drop-shadow");
                    let yaml = yaml_doc.pop().unwrap();
                    Some(FilterOp::DropShadow(Shadow {
                        offset: yaml["offset"].as_vector().unwrap(),
                        blur_radius: yaml["blur-radius"].as_f32().unwrap(),
                        color: yaml["color"].as_colorf().unwrap()
                    }))
                }
                ("color-matrix", ref args, _) if args.len() == 20 => {
                    let m: Vec<f32> = args.iter().map(|f| f.parse().unwrap()).collect();
                    let mut matrix: [f32; 20] = [0.0; 20];
                    matrix.clone_from_slice(&m);
                    Some(FilterOp::ColorMatrix(matrix))
                }
                ("flood", ref args, _) if args.len() == 1 => {
                    let str = format!("---\ncolor: {}\n", args[0]);
                    let mut yaml_doc = YamlLoader::load_from_str(&str).expect("Failed to parse flood");
                    let yaml = yaml_doc.pop().unwrap();
                    Some(FilterOp::Flood(yaml["color"].as_colorf().unwrap()))
                }
                (_, _, _) => None,
            }
        } else {
            None
        }
    }

    fn as_vec_filter_op(&self) -> Option<Vec<FilterOp>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|x| x.as_filter_op().unwrap()).collect())
        } else {
            self.as_filter_op().map(|op| vec![op])
        }
    }

    fn as_filter_data(&self) -> Option<FilterData> {
        // Parse an array with five entries. First entry is an array of func types (4).
        // The remaining entries are arrays of floats.
        if let Yaml::Array(ref array) = *self {
            if array.len() != 5 {
                panic!("Invalid filter data specified, base array doesn't have five entries: {:?}", self);
            }
            if let Some(func_types_p) = array[0].as_vec_string() {
                if func_types_p.len() != 4 {
                    panic!("Invalid filter data specified, func type array doesn't have five entries: {:?}", self);
                }
                let func_types: Vec<ComponentTransferFuncType> =
                    func_types_p.into_iter().map(|x| { match StringEnum::from_str(&x) {
                        Some(y) => y,
                        None => panic!("Invalid filter data specified, invalid func type name: {:?}", self),
                    }}).collect();
                if let Some(r_values_p) = array[1].as_vec_f32() {
                    if let Some(g_values_p) = array[2].as_vec_f32() {
                        if let Some(b_values_p) = array[3].as_vec_f32() {
                            if let Some(a_values_p) = array[4].as_vec_f32() {
                                let filter_data = FilterData {
                                    func_r_type: func_types[0],
                                    r_values: r_values_p,
                                    func_g_type: func_types[1],
                                    g_values: g_values_p,
                                    func_b_type: func_types[2],
                                    b_values: b_values_p,
                                    func_a_type: func_types[3],
                                    a_values: a_values_p,
                                };
                                return Some(filter_data)
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn as_filter_input(&self) -> Option<FilterPrimitiveInput> {
        if let Some(input) = self.as_str() {
            match input {
                "original" => Some(FilterPrimitiveInput::Original),
                "previous" => Some(FilterPrimitiveInput::Previous),
                _ => None,
            }
        } else if let Some(index) = self.as_i64() {
            if index >= 0 {
                Some(FilterPrimitiveInput::OutputOfPrimitiveIndex(index as usize))
            } else {
                panic!("Filter input index cannot be negative");
            }
        } else {
            panic!("Invalid filter input");
        }
    }

    fn as_vec_filter_data(&self) -> Option<Vec<FilterData>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|x| x.as_filter_data().unwrap()).collect())
        } else {
            self.as_filter_data().map(|data| vec![data])
        }
    }

    fn as_filter_primitive(&self) -> Option<FilterPrimitive> {
        if let Some(filter_type) = self["type"].as_str() {
            let kind = match filter_type {
                "identity" => {
                    FilterPrimitiveKind::Identity(IdentityPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                    })
                }
                "blend" => {
                    FilterPrimitiveKind::Blend(BlendPrimitive {
                        input1: self["in1"].as_filter_input().unwrap(),
                        input2: self["in2"].as_filter_input().unwrap(),
                        mode: self["blend-mode"].as_mix_blend_mode().unwrap(),
                    })
                }
                "flood" => {
                    FilterPrimitiveKind::Flood(FloodPrimitive {
                        color: self["color"].as_colorf().unwrap(),
                    })
                }
                "blur" => {
                    FilterPrimitiveKind::Blur(BlurPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                        radius: self["radius"].as_f32().unwrap(),
                    })
                }
                "opacity" => {
                    FilterPrimitiveKind::Opacity(OpacityPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                        opacity: self["opacity"].as_f32().unwrap(),
                    })
                }
                "color-matrix" => {
                    let m: Vec<f32> = self["matrix"].as_vec_f32().unwrap();
                    let mut matrix: [f32; 20] = [0.0; 20];
                    matrix.clone_from_slice(&m);

                    FilterPrimitiveKind::ColorMatrix(ColorMatrixPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                        matrix,
                    })
                }
                "drop-shadow" => {
                    FilterPrimitiveKind::DropShadow(DropShadowPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                        shadow: Shadow {
                            offset: self["offset"].as_vector().unwrap(),
                            color: self["color"].as_colorf().unwrap(),
                            blur_radius: self["radius"].as_f32().unwrap(),
                        }
                    })
                }
                "component-transfer" => {
                    FilterPrimitiveKind::ComponentTransfer(ComponentTransferPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                    })
                }
                "offset" => {
                    FilterPrimitiveKind::Offset(OffsetPrimitive {
                        input: self["in"].as_filter_input().unwrap(),
                        offset: self["offset"].as_vector().unwrap(),
                    })
                }
                "composite" => {
                    let operator = match self["operator"].as_str().unwrap() {
                        "over" => CompositeOperator::Over,
                        "in" => CompositeOperator::In,
                        "out" => CompositeOperator::Out,
                        "atop" => CompositeOperator::Atop,
                        "xor" => CompositeOperator::Xor,
                        "lighter" => CompositeOperator::Lighter,
                        "arithmetic" => {
                            let k_vals = self["k-values"].as_vec_f32().unwrap();
                            assert!(k_vals.len() == 4, "Must be 4 k values for arithmetic composite operator");
                            let k_vals = [k_vals[0], k_vals[1], k_vals[2], k_vals[3]];
                            CompositeOperator::Arithmetic(k_vals)
                        }
                        _ => panic!("Invalid composite operator"),
                    };
                    FilterPrimitiveKind::Composite(CompositePrimitive {
                        input1: self["in1"].as_filter_input().unwrap(),
                        input2: self["in2"].as_filter_input().unwrap(),
                        operator,
                    })
                }
                _ => return None,
            };

            Some(FilterPrimitive {
                kind,
                color_space: self["color-space"].as_color_space().unwrap_or(ColorSpace::LinearRgb),
            })
        } else {
            None
        }
    }

    fn as_vec_filter_primitive(&self) -> Option<Vec<FilterPrimitive>> {
        if let Some(v) = self.as_vec() {
            Some(v.iter().map(|x| x.as_filter_primitive().unwrap()).collect())
        } else {
            self.as_filter_primitive().map(|data| vec![data])
        }
    }

    fn as_color_space(&self) -> Option<ColorSpace> {
        self.as_str().and_then(|x| StringEnum::from_str(x))
    }
}
