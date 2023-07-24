/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use glsl::syntax;
use glsl::syntax::{ArrayedIdentifier, ArraySpecifier, AssignmentOp, BinaryOp, Identifier};
use glsl::syntax::{NonEmpty, PrecisionQualifier, StructFieldSpecifier, StructSpecifier};
use glsl::syntax::{TypeSpecifier, TypeSpecifierNonArray, UnaryOp};
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

trait LiftFrom<S> {
    fn lift(state: &mut State, s: S) -> Self;
}

fn lift<S, T: LiftFrom<S>>(state: &mut State, s: S) -> T {
    LiftFrom::lift(state, s)
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub decl: SymDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    ret: Type,
    params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    signatures: NonEmpty<FunctionSignature>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplerFormat {
    Unknown,
    RGBA8,
    RGBA32F,
    RGBA32I,
    R8,
}

impl SamplerFormat {
    pub fn type_suffix(self) -> Option<&'static str> {
        match self {
            SamplerFormat::Unknown => None,
            SamplerFormat::RGBA8 => Some("RGBA8"),
            SamplerFormat::RGBA32F => Some("RGBA32F"),
            SamplerFormat::RGBA32I => Some("RGBA32I"),
            SamplerFormat::R8 => Some("R8"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StorageClass {
    None,
    Const,
    In,
    Out,
    Uniform,
    Sampler(SamplerFormat),
    FragColor(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArraySizes {
    pub sizes: Vec<Expr>,
}

impl LiftFrom<&ArraySpecifier> for ArraySizes {
    fn lift(state: &mut State, a: &ArraySpecifier) -> Self {
        ArraySizes {
            sizes: vec![match a {
                ArraySpecifier::Unsized => panic!(),
                ArraySpecifier::ExplicitlySized(expr) => translate_expression(state, expr),
            }],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TypeKind {
    Void,
    Bool,
    Int,
    UInt,
    Float,
    Double,
    Vec2,
    Vec3,
    Vec4,
    DVec2,
    DVec3,
    DVec4,
    BVec2,
    BVec3,
    BVec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat2,
    Mat3,
    Mat4,
    Mat23,
    Mat24,
    Mat32,
    Mat34,
    Mat42,
    Mat43,
    DMat2,
    DMat3,
    DMat4,
    DMat23,
    DMat24,
    DMat32,
    DMat34,
    DMat42,
    DMat43,
    // floating point opaque types
    Sampler1D,
    Image1D,
    Sampler2D,
    Image2D,
    Sampler3D,
    Image3D,
    SamplerCube,
    ImageCube,
    Sampler2DRect,
    Image2DRect,
    Sampler1DArray,
    Image1DArray,
    Sampler2DArray,
    Image2DArray,
    SamplerBuffer,
    ImageBuffer,
    Sampler2DMS,
    Image2DMS,
    Sampler2DMSArray,
    Image2DMSArray,
    SamplerCubeArray,
    ImageCubeArray,
    Sampler1DShadow,
    Sampler2DShadow,
    Sampler2DRectShadow,
    Sampler1DArrayShadow,
    Sampler2DArrayShadow,
    SamplerCubeShadow,
    SamplerCubeArrayShadow,
    // signed integer opaque types
    ISampler1D,
    IImage1D,
    ISampler2D,
    IImage2D,
    ISampler3D,
    IImage3D,
    ISamplerCube,
    IImageCube,
    ISampler2DRect,
    IImage2DRect,
    ISampler1DArray,
    IImage1DArray,
    ISampler2DArray,
    IImage2DArray,
    ISamplerBuffer,
    IImageBuffer,
    ISampler2DMS,
    IImage2DMS,
    ISampler2DMSArray,
    IImage2DMSArray,
    ISamplerCubeArray,
    IImageCubeArray,
    // unsigned integer opaque types
    AtomicUInt,
    USampler1D,
    UImage1D,
    USampler2D,
    UImage2D,
    USampler3D,
    UImage3D,
    USamplerCube,
    UImageCube,
    USampler2DRect,
    UImage2DRect,
    USampler1DArray,
    UImage1DArray,
    USampler2DArray,
    UImage2DArray,
    USamplerBuffer,
    UImageBuffer,
    USampler2DMS,
    UImage2DMS,
    USampler2DMSArray,
    UImage2DMSArray,
    USamplerCubeArray,
    UImageCubeArray,
    Struct(SymRef),
}

impl TypeKind {
    pub fn is_sampler(&self) -> bool {
        use TypeKind::*;
        match self {
            Sampler1D
            | Image1D
            | Sampler2D
            | Image2D
            | Sampler3D
            | Image3D
            | SamplerCube
            | ImageCube
            | Sampler2DRect
            | Image2DRect
            | Sampler1DArray
            | Image1DArray
            | Sampler2DArray
            | Image2DArray
            | SamplerBuffer
            | ImageBuffer
            | Sampler2DMS
            | Image2DMS
            | Sampler2DMSArray
            | Image2DMSArray
            | SamplerCubeArray
            | ImageCubeArray
            | Sampler1DShadow
            | Sampler2DShadow
            | Sampler2DRectShadow
            | Sampler1DArrayShadow
            | Sampler2DArrayShadow
            | SamplerCubeShadow
            | SamplerCubeArrayShadow
            | ISampler1D
            | IImage1D
            | ISampler2D
            | IImage2D
            | ISampler3D
            | IImage3D
            | ISamplerCube
            | IImageCube
            | ISampler2DRect
            | IImage2DRect
            | ISampler1DArray
            | IImage1DArray
            | ISampler2DArray
            | IImage2DArray
            | ISamplerBuffer
            | IImageBuffer
            | ISampler2DMS
            | IImage2DMS
            | ISampler2DMSArray
            | IImage2DMSArray
            | ISamplerCubeArray
            | IImageCubeArray
            | USampler1D
            | UImage1D
            | USampler2D
            | UImage2D
            | USampler3D
            | UImage3D
            | USamplerCube
            | UImageCube
            | USampler2DRect
            | UImage2DRect
            | USampler1DArray
            | UImage1DArray
            | USampler2DArray
            | UImage2DArray
            | USamplerBuffer
            | UImageBuffer
            | USampler2DMS
            | UImage2DMS
            | USampler2DMSArray
            | UImage2DMSArray
            | USamplerCubeArray
            | UImageCubeArray => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        use TypeKind::*;
        match self {
            Bool | BVec2 | BVec3 | BVec4 => true,
            _ => false,
        }
    }

    pub fn to_bool(&self) -> Self {
        use TypeKind::*;
        match self {
            Int | UInt | Float | Double => Bool,
            IVec2 | UVec2 | Vec2 | DVec2 => BVec2,
            IVec3 | UVec3 | Vec3 | DVec3 => BVec3,
            IVec4 | UVec4 | Vec4 | DVec4 => BVec4,
            _ => *self,
        }
    }

    pub fn to_int(&self) -> Self {
        use TypeKind::*;
        match self {
            Bool | UInt | Float | Double => Int,
            BVec2 | UVec2 | Vec2 | DVec2 => IVec2,
            BVec3 | UVec3 | Vec3 | DVec3 => IVec3,
            BVec4 | UVec4 | Vec4 | DVec4 => IVec4,
            _ => *self,
        }
    }

    pub fn to_scalar(&self) -> Self {
        use TypeKind::*;
        match self {
            IVec2 | IVec3 | IVec4 => Int,
            UVec2 | UVec3 | UVec4 => UInt,
            Vec2 | Vec3 | Vec4 => Float,
            DVec2 | DVec3 | DVec4 => Double,
            BVec2 | BVec3 | BVec4 => Bool,
            _ => *self,
        }
    }

    pub fn glsl_primitive_type_name(&self) -> Option<&'static str> {
        use TypeKind::*;
        Some(match self {
            Void => "void",
            Bool => "bool",
            Int => "int",
            UInt => "uint",
            Float => "float",
            Double => "double",
            Vec2 => "vec2",
            Vec3 => "vec3",
            Vec4 => "vec4",
            DVec2 => "dvec2",
            DVec3 => "dvec3",
            DVec4 => "dvec4",
            BVec2 => "bvec2",
            BVec3 => "bvec3",
            BVec4 => "bvec4",
            IVec2 => "ivec2",
            IVec3 => "ivec3",
            IVec4 => "ivec4",
            UVec2 => "uvec2",
            UVec3 => "uvec3",
            UVec4 => "uvec4",
            Mat2 => "mat2",
            Mat3 => "mat3",
            Mat4 => "mat4",
            Mat23 => "mat23",
            Mat24 => "mat24",
            Mat32 => "mat32",
            Mat34 => "mat34",
            Mat42 => "mat42",
            Mat43 => "mat43",
            DMat2 => "dmat2",
            DMat3 => "dmat3",
            DMat4 => "dmat4",
            DMat23 => "dmat23",
            DMat24 => "dmat24",
            DMat32 => "dmat32",
            DMat34 => "dmat34",
            DMat42 => "dmat42",
            DMat43 => "dmat43",
            Sampler1D => "sampler1D",
            Image1D => "image1D",
            Sampler2D => "sampler2D",
            Image2D => "image2D",
            Sampler3D => "sampler3D",
            Image3D => "image3D",
            SamplerCube => "samplerCube",
            ImageCube => "imageCube",
            Sampler2DRect => "sampler2DRect",
            Image2DRect => "image2DRect",
            Sampler1DArray => "sampler1DArray",
            Image1DArray => "image1DArray",
            Sampler2DArray => "sampler2DArray",
            Image2DArray => "image2DArray",
            SamplerBuffer => "samplerBuffer",
            ImageBuffer => "imageBuffer",
            Sampler2DMS => "sampler2DMS",
            Image2DMS => "image2DMS",
            Sampler2DMSArray => "sampler2DMSArray",
            Image2DMSArray => "image2DMSArray",
            SamplerCubeArray => "samplerCubeArray",
            ImageCubeArray => "imageCubeArray",
            Sampler1DShadow => "sampler1DShadow",
            Sampler2DShadow => "sampler2DShadow",
            Sampler2DRectShadow => "sampler2DRectShadow",
            Sampler1DArrayShadow => "sampler1DArrayShadow",
            Sampler2DArrayShadow => "sampler2DArrayShadow",
            SamplerCubeShadow => "samplerCubeShadow",
            SamplerCubeArrayShadow => "samplerCubeArrayShadow",
            ISampler1D => "isampler1D",
            IImage1D => "iimage1D",
            ISampler2D => "isampler2D",
            IImage2D => "iimage2D",
            ISampler3D => "isampler3D",
            IImage3D => "iimage3D",
            ISamplerCube => "isamplerCube",
            IImageCube => "iimageCube",
            ISampler2DRect => "isampler2DRect",
            IImage2DRect => "iimage2DRect",
            ISampler1DArray => "isampler1DArray",
            IImage1DArray => "iimage1DArray",
            ISampler2DArray => "isampler2DArray",
            IImage2DArray => "iimage2DArray",
            ISamplerBuffer => "isamplerBuffer",
            IImageBuffer => "iimageBuffer",
            ISampler2DMS => "isampler2MS",
            IImage2DMS => "iimage2DMS",
            ISampler2DMSArray => "isampler2DMSArray",
            IImage2DMSArray => "iimage2DMSArray",
            ISamplerCubeArray => "isamplerCubeArray",
            IImageCubeArray => "iimageCubeArray",
            AtomicUInt => "atomic_uint",
            USampler1D => "usampler1D",
            UImage1D => "uimage1D",
            USampler2D => "usampler2D",
            UImage2D => "uimage2D",
            USampler3D => "usampler3D",
            UImage3D => "uimage3D",
            USamplerCube => "usamplerCube",
            UImageCube => "uimageCube",
            USampler2DRect => "usampler2DRect",
            UImage2DRect => "uimage2DRect",
            USampler1DArray => "usampler1DArray",
            UImage1DArray => "uimage1DArray",
            USampler2DArray => "usampler2DArray",
            UImage2DArray => "uimage2DArray",
            USamplerBuffer => "usamplerBuffer",
            UImageBuffer => "uimageBuffer",
            USampler2DMS => "usampler2DMS",
            UImage2DMS => "uimage2DMS",
            USampler2DMSArray => "usamplerDMSArray",
            UImage2DMSArray => "uimage2DMSArray",
            USamplerCubeArray => "usamplerCubeArray",
            UImageCubeArray => "uimageCubeArray",
            Struct(..) => return None,
        })
    }

    pub fn cxx_primitive_type_name(&self) -> Option<&'static str> {
        use TypeKind::*;
        match self {
            Bool => Some("Bool"),
            Int => Some("I32"),
            UInt => Some("U32"),
            Float => Some("Float"),
            Double => Some("Double"),
            _ => self.glsl_primitive_type_name(),
        }
    }

    pub fn cxx_primitive_scalar_type_name(&self) -> Option<&'static str> {
        use TypeKind::*;
        match self {
            Void => Some("void"),
            Bool => Some("bool"),
            Int => Some("int32_t"),
            UInt => Some("uint32_t"),
            Float => Some("float"),
            Double => Some("double"),
            _ => {
                if self.is_sampler() {
                    self.cxx_primitive_type_name()
                } else {
                    None
                }
            }
        }
    }

    pub fn from_glsl_primitive_type_name(name: &str) -> Option<TypeKind> {
        use TypeKind::*;
        Some(match name {
            "void" => Void,
            "bool" => Bool,
            "int" => Int,
            "uint" => UInt,
            "float" => Float,
            "double" => Double,
            "vec2" => Vec2,
            "vec3" => Vec3,
            "vec4" => Vec4,
            "dvec2" => DVec2,
            "dvec3" => DVec3,
            "dvec4" => DVec4,
            "bvec2" => BVec2,
            "bvec3" => BVec3,
            "bvec4" => BVec4,
            "ivec2" => IVec2,
            "ivec3" => IVec3,
            "ivec4" => IVec4,
            "uvec2" => UVec2,
            "uvec3" => UVec3,
            "uvec4" => UVec4,
            "mat2" => Mat2,
            "mat3" => Mat3,
            "mat4" => Mat4,
            "mat23" => Mat23,
            "mat24" => Mat24,
            "mat32" => Mat32,
            "mat34" => Mat34,
            "mat42" => Mat42,
            "mat43" => Mat43,
            "dmat2" => DMat2,
            "dmat3" => DMat3,
            "dmat4" => DMat4,
            "dmat23" => DMat23,
            "dmat24" => DMat24,
            "dmat32" => DMat32,
            "dmat34" => DMat34,
            "dmat42" => DMat42,
            "dmat43" => DMat43,
            "sampler1D" => Sampler1D,
            "image1D" => Image1D,
            "sampler2D" => Sampler2D,
            "image2D" => Image2D,
            "sampler3D" => Sampler3D,
            "image3D" => Image3D,
            "samplerCube" => SamplerCube,
            "imageCube" => ImageCube,
            "sampler2DRect" => Sampler2DRect,
            "image2DRect" => Image2DRect,
            "sampler1DArray" => Sampler1DArray,
            "image1DArray" => Image1DArray,
            "sampler2DArray" => Sampler2DArray,
            "image2DArray" => Image2DArray,
            "samplerBuffer" => SamplerBuffer,
            "imageBuffer" => ImageBuffer,
            "sampler2DMS" => Sampler2DMS,
            "image2DMS" => Image2DMS,
            "sampler2DMSArray" => Sampler2DMSArray,
            "image2DMSArray" => Image2DMSArray,
            "samplerCubeArray" => SamplerCubeArray,
            "imageCubeArray" => ImageCubeArray,
            "sampler1DShadow" => Sampler1DShadow,
            "sampler2DShadow" => Sampler2DShadow,
            "sampler2DRectShadow" => Sampler2DRectShadow,
            "sampler1DArrayShadow" => Sampler1DArrayShadow,
            "sampler2DArrayShadow" => Sampler2DArrayShadow,
            "samplerCubeShadow" => SamplerCubeShadow,
            "samplerCubeArrayShadow" => SamplerCubeArrayShadow,
            "isampler1D" => ISampler1D,
            "iimage1D" => IImage1D,
            "isampler2D" => ISampler2D,
            "iimage2D" => IImage2D,
            "isampler3D" => ISampler3D,
            "iimage3D" => IImage3D,
            "isamplerCube" => ISamplerCube,
            "iimageCube" => IImageCube,
            "isampler2DRect" => ISampler2DRect,
            "iimage2DRect" => IImage2DRect,
            "isampler1DArray" => ISampler1DArray,
            "iimage1DArray" => IImage1DArray,
            "isampler2DArray" => ISampler2DArray,
            "iimage2DArray" => IImage2DArray,
            "isamplerBuffer" => ISamplerBuffer,
            "iimageBuffer" => IImageBuffer,
            "isampler2MS" => ISampler2DMS,
            "iimage2DMS" => IImage2DMS,
            "isampler2DMSArray" => ISampler2DMSArray,
            "iimage2DMSArray" => IImage2DMSArray,
            "isamplerCubeArray" => ISamplerCubeArray,
            "iimageCubeArray" => IImageCubeArray,
            "atomic_uint" => AtomicUInt,
            "usampler1D" => USampler1D,
            "uimage1D" => UImage1D,
            "usampler2D" => USampler2D,
            "uimage2D" => UImage2D,
            "usampler3D" => USampler3D,
            "uimage3D" => UImage3D,
            "usamplerCube" => USamplerCube,
            "uimageCube" => UImageCube,
            "usampler2DRect" => USampler2DRect,
            "uimage2DRect" => UImage2DRect,
            "usampler1DArray" => USampler1DArray,
            "uimage1DArray" => UImage1DArray,
            "usampler2DArray" => USampler2DArray,
            "uimage2DArray" => UImage2DArray,
            "usamplerBuffer" => USamplerBuffer,
            "uimageBuffer" => UImageBuffer,
            "usampler2DMS" => USampler2DMS,
            "uimage2DMS" => UImage2DMS,
            "usamplerDMSArray" => USampler2DMSArray,
            "uimage2DMSArray" => UImage2DMSArray,
            "usamplerCubeArray" => USamplerCubeArray,
            "uimageCubeArray" => UImageCubeArray,
            _ => return None,
        })
    }

    pub fn from_primitive_type_specifier(spec: &syntax::TypeSpecifierNonArray) -> Option<TypeKind> {
        use TypeKind::*;
        Some(match spec {
            TypeSpecifierNonArray::Void => Void,
            TypeSpecifierNonArray::Bool => Bool,
            TypeSpecifierNonArray::Int => Int,
            TypeSpecifierNonArray::UInt => UInt,
            TypeSpecifierNonArray::Float => Float,
            TypeSpecifierNonArray::Double => Double,
            TypeSpecifierNonArray::Vec2 => Vec2,
            TypeSpecifierNonArray::Vec3 => Vec3,
            TypeSpecifierNonArray::Vec4 => Vec4,
            TypeSpecifierNonArray::DVec2 => DVec2,
            TypeSpecifierNonArray::DVec3 => DVec3,
            TypeSpecifierNonArray::DVec4 => DVec4,
            TypeSpecifierNonArray::BVec2 => BVec2,
            TypeSpecifierNonArray::BVec3 => BVec3,
            TypeSpecifierNonArray::BVec4 => BVec4,
            TypeSpecifierNonArray::IVec2 => IVec2,
            TypeSpecifierNonArray::IVec3 => IVec3,
            TypeSpecifierNonArray::IVec4 => IVec4,
            TypeSpecifierNonArray::UVec2 => UVec2,
            TypeSpecifierNonArray::UVec3 => UVec3,
            TypeSpecifierNonArray::UVec4 => UVec4,
            TypeSpecifierNonArray::Mat2 => Mat2,
            TypeSpecifierNonArray::Mat3 => Mat3,
            TypeSpecifierNonArray::Mat4 => Mat4,
            TypeSpecifierNonArray::Mat23 => Mat23,
            TypeSpecifierNonArray::Mat24 => Mat24,
            TypeSpecifierNonArray::Mat32 => Mat32,
            TypeSpecifierNonArray::Mat34 => Mat34,
            TypeSpecifierNonArray::Mat42 => Mat42,
            TypeSpecifierNonArray::Mat43 => Mat43,
            TypeSpecifierNonArray::DMat2 => DMat2,
            TypeSpecifierNonArray::DMat3 => DMat3,
            TypeSpecifierNonArray::DMat4 => DMat4,
            TypeSpecifierNonArray::DMat23 => DMat23,
            TypeSpecifierNonArray::DMat24 => DMat24,
            TypeSpecifierNonArray::DMat32 => DMat32,
            TypeSpecifierNonArray::DMat34 => DMat34,
            TypeSpecifierNonArray::DMat42 => DMat42,
            TypeSpecifierNonArray::DMat43 => DMat43,
            TypeSpecifierNonArray::Sampler1D => Sampler1D,
            TypeSpecifierNonArray::Image1D => Image1D,
            TypeSpecifierNonArray::Sampler2D => Sampler2D,
            TypeSpecifierNonArray::Image2D => Image2D,
            TypeSpecifierNonArray::Sampler3D => Sampler3D,
            TypeSpecifierNonArray::Image3D => Image3D,
            TypeSpecifierNonArray::SamplerCube => SamplerCube,
            TypeSpecifierNonArray::ImageCube => ImageCube,
            TypeSpecifierNonArray::Sampler2DRect => Sampler2DRect,
            TypeSpecifierNonArray::Image2DRect => Image2DRect,
            TypeSpecifierNonArray::Sampler1DArray => Sampler1DArray,
            TypeSpecifierNonArray::Image1DArray => Image1DArray,
            TypeSpecifierNonArray::Sampler2DArray => Sampler2DArray,
            TypeSpecifierNonArray::Image2DArray => Image2DArray,
            TypeSpecifierNonArray::SamplerBuffer => SamplerBuffer,
            TypeSpecifierNonArray::ImageBuffer => ImageBuffer,
            TypeSpecifierNonArray::Sampler2DMS => Sampler2DMS,
            TypeSpecifierNonArray::Image2DMS => Image2DMS,
            TypeSpecifierNonArray::Sampler2DMSArray => Sampler2DMSArray,
            TypeSpecifierNonArray::Image2DMSArray => Image2DMSArray,
            TypeSpecifierNonArray::SamplerCubeArray => SamplerCubeArray,
            TypeSpecifierNonArray::ImageCubeArray => ImageCubeArray,
            TypeSpecifierNonArray::Sampler1DShadow => Sampler1DShadow,
            TypeSpecifierNonArray::Sampler2DShadow => Sampler2DShadow,
            TypeSpecifierNonArray::Sampler2DRectShadow => Sampler2DRectShadow,
            TypeSpecifierNonArray::Sampler1DArrayShadow => Sampler1DArrayShadow,
            TypeSpecifierNonArray::Sampler2DArrayShadow => Sampler2DArrayShadow,
            TypeSpecifierNonArray::SamplerCubeShadow => SamplerCubeShadow,
            TypeSpecifierNonArray::SamplerCubeArrayShadow => SamplerCubeArrayShadow,
            TypeSpecifierNonArray::ISampler1D => ISampler1D,
            TypeSpecifierNonArray::IImage1D => IImage1D,
            TypeSpecifierNonArray::ISampler2D => ISampler2D,
            TypeSpecifierNonArray::IImage2D => IImage2D,
            TypeSpecifierNonArray::ISampler3D => ISampler3D,
            TypeSpecifierNonArray::IImage3D => IImage3D,
            TypeSpecifierNonArray::ISamplerCube => ISamplerCube,
            TypeSpecifierNonArray::IImageCube => IImageCube,
            TypeSpecifierNonArray::ISampler2DRect => ISampler2DRect,
            TypeSpecifierNonArray::IImage2DRect => IImage2DRect,
            TypeSpecifierNonArray::ISampler1DArray => ISampler1DArray,
            TypeSpecifierNonArray::IImage1DArray => IImage1DArray,
            TypeSpecifierNonArray::ISampler2DArray => ISampler2DArray,
            TypeSpecifierNonArray::IImage2DArray => IImage2DArray,
            TypeSpecifierNonArray::ISamplerBuffer => ISamplerBuffer,
            TypeSpecifierNonArray::IImageBuffer => IImageBuffer,
            TypeSpecifierNonArray::ISampler2DMS => ISampler2DMS,
            TypeSpecifierNonArray::IImage2DMS => IImage2DMS,
            TypeSpecifierNonArray::ISampler2DMSArray => ISampler2DMSArray,
            TypeSpecifierNonArray::IImage2DMSArray => IImage2DMSArray,
            TypeSpecifierNonArray::ISamplerCubeArray => ISamplerCubeArray,
            TypeSpecifierNonArray::IImageCubeArray => IImageCubeArray,
            TypeSpecifierNonArray::AtomicUInt => AtomicUInt,
            TypeSpecifierNonArray::USampler1D => USampler1D,
            TypeSpecifierNonArray::UImage1D => UImage1D,
            TypeSpecifierNonArray::USampler2D => USampler2D,
            TypeSpecifierNonArray::UImage2D => UImage2D,
            TypeSpecifierNonArray::USampler3D => USampler3D,
            TypeSpecifierNonArray::UImage3D => UImage3D,
            TypeSpecifierNonArray::USamplerCube => USamplerCube,
            TypeSpecifierNonArray::UImageCube => UImageCube,
            TypeSpecifierNonArray::USampler2DRect => USampler2DRect,
            TypeSpecifierNonArray::UImage2DRect => UImage2DRect,
            TypeSpecifierNonArray::USampler1DArray => USampler1DArray,
            TypeSpecifierNonArray::UImage1DArray => UImage1DArray,
            TypeSpecifierNonArray::USampler2DArray => USampler2DArray,
            TypeSpecifierNonArray::UImage2DArray => UImage2DArray,
            TypeSpecifierNonArray::USamplerBuffer => USamplerBuffer,
            TypeSpecifierNonArray::UImageBuffer => UImageBuffer,
            TypeSpecifierNonArray::USampler2DMS => USampler2DMS,
            TypeSpecifierNonArray::UImage2DMS => UImage2DMS,
            TypeSpecifierNonArray::USampler2DMSArray => USampler2DMSArray,
            TypeSpecifierNonArray::UImage2DMSArray => UImage2DMSArray,
            TypeSpecifierNonArray::USamplerCubeArray => USamplerCubeArray,
            TypeSpecifierNonArray::UImageCubeArray => UImageCubeArray,
            TypeSpecifierNonArray::Struct(..) | TypeSpecifierNonArray::TypeName(..) => return None,
        })
    }
}

impl LiftFrom<&syntax::TypeSpecifierNonArray> for TypeKind {
    fn lift(state: &mut State, spec: &syntax::TypeSpecifierNonArray) -> Self {
        use TypeKind::*;
        if let Some(kind) = TypeKind::from_primitive_type_specifier(spec) {
            kind
        } else {
            match spec {
                TypeSpecifierNonArray::Struct(s) => {
                    Struct(state.lookup(s.name.as_ref().unwrap().as_str()).unwrap())
                }
                TypeSpecifierNonArray::TypeName(s) => Struct(state.lookup(&s.0).unwrap()),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub precision: Option<PrecisionQualifier>,
    pub array_sizes: Option<Box<ArraySizes>>,
}

impl Type {
    pub fn new(kind: TypeKind) -> Self {
        Type {
            kind,
            precision: None,
            array_sizes: None,
        }
    }
}

impl LiftFrom<&syntax::FullySpecifiedType> for Type {
    fn lift(state: &mut State, ty: &syntax::FullySpecifiedType) -> Self {
        let kind = lift(state, &ty.ty.ty);
        let array_sizes = match ty.ty.array_specifier.as_ref() {
            Some(x) => Some(Box::new(lift(state, x))),
            None => None,
        };
        let precision = get_precision(&ty.qualifier);
        Type {
            kind,
            precision,
            array_sizes,
        }
    }
}

impl LiftFrom<&syntax::TypeSpecifier> for Type {
    fn lift(state: &mut State, ty: &syntax::TypeSpecifier) -> Self {
        let kind = lift(state, &ty.ty);
        let array_sizes = ty
            .array_specifier
            .as_ref()
            .map(|x| Box::new(lift(state, x)));
        Type {
            kind,
            precision: None,
            array_sizes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub ty: Type,
    pub name: syntax::Identifier,
}

fn get_precision(qualifiers: &Option<syntax::TypeQualifier>) -> Option<PrecisionQualifier> {
    let mut precision = None;
    for qual in qualifiers.iter().flat_map(|x| x.qualifiers.0.iter()) {
        match qual {
            syntax::TypeQualifierSpec::Precision(p) => {
                if precision.is_some() {
                    panic!("Multiple precisions");
                }
                precision = Some(p.clone());
            }
            _ => {}
        }
    }
    precision
}

impl LiftFrom<&StructFieldSpecifier> for StructField {
    fn lift(state: &mut State, f: &StructFieldSpecifier) -> Self {
        let mut ty: Type = lift(state, &f.ty);
        match &f.identifiers.0[..] {
            [ident] => {
                if let Some(a) = &ident.array_spec {
                    ty.array_sizes = Some(Box::new(lift(state, a)));
                }
                StructField {
                    ty,
                    name: ident.ident.clone(),
                }
            }
            _ => panic!("bad number of identifiers"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructFields {
    pub fields: Vec<StructField>,
}

impl LiftFrom<&StructSpecifier> for StructFields {
    fn lift(state: &mut State, s: &StructSpecifier) -> Self {
        let fields = s.fields.0.iter().map(|field| lift(state, field)).collect();
        Self { fields }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunClass {
    Unknown,
    Scalar,
    Vector,
    Dependent(u32),
}

impl RunClass {
    pub fn merge(self, run_class: RunClass) -> RunClass {
        match (self, run_class) {
            (RunClass::Vector, _) | (_, RunClass::Vector) => RunClass::Vector,
            (RunClass::Dependent(x), RunClass::Dependent(y)) => RunClass::Dependent(x | y),
            (RunClass::Unknown, _) | (_, RunClass::Dependent(..)) => run_class,
            _ => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymDecl {
    NativeFunction(FunctionType, Option<&'static str>),
    UserFunction(Rc<FunctionDefinition>, RunClass),
    Local(StorageClass, Type, RunClass),
    Global(
        StorageClass,
        Option<syntax::InterpolationQualifier>,
        Type,
        RunClass,
    ),
    Struct(StructFields),
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, Hash)]
pub struct SymRef(u32);

#[derive(Debug)]
struct Scope {
    name: String,
    names: HashMap<String, SymRef>,
}
impl Scope {
    fn new(name: String) -> Self {
        Scope {
            name,
            names: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexelFetchOffsets {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl TexelFetchOffsets {
    fn new(x: i32, y: i32) -> Self {
        TexelFetchOffsets {
            min_x: x,
            max_x: x,
            min_y: y,
            max_y: y,
        }
    }

    fn add_offset(&mut self, x: i32, y: i32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }
}

#[derive(Debug)]
pub struct State {
    scopes: Vec<Scope>,
    syms: Vec<RefCell<Symbol>>,
    in_function: Option<SymRef>,
    run_class_changed: Cell<bool>,
    last_declaration: SymRef,
    branch_run_class: RunClass,
    branch_declaration: SymRef,
    modified_globals: RefCell<Vec<SymRef>>,
    pub used_globals: RefCell<Vec<SymRef>>,
    pub texel_fetches: HashMap<(SymRef, SymRef), TexelFetchOffsets>,
}

impl State {
    pub fn new() -> Self {
        State {
            scopes: Vec::new(),
            syms: Vec::new(),
            in_function: None,
            run_class_changed: Cell::new(false),
            last_declaration: SymRef(0),
            branch_run_class: RunClass::Unknown,
            branch_declaration: SymRef(0),
            modified_globals: RefCell::new(Vec::new()),
            used_globals: RefCell::new(Vec::new()),
            texel_fetches: HashMap::new(),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<SymRef> {
        for s in self.scopes.iter().rev() {
            if let Some(sym) = s.names.get(name) {
                return Some(*sym);
            }
        }
        return None;
    }

    fn declare(&mut self, name: &str, decl: SymDecl) -> SymRef {
        let s = SymRef(self.syms.len() as u32);
        self.syms.push(RefCell::new(Symbol {
            name: name.into(),
            decl,
        }));
        self.scopes.last_mut().unwrap().names.insert(name.into(), s);
        s
    }

    pub fn sym(&self, sym: SymRef) -> Ref<Symbol> {
        self.syms[sym.0 as usize].borrow()
    }

    pub fn sym_mut(&mut self, sym: SymRef) -> &mut Symbol {
        self.syms[sym.0 as usize].get_mut()
    }

    pub fn lookup_sym_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.lookup(name)
            .map(move |x| self.syms[x.0 as usize].get_mut())
    }

    fn push_scope(&mut self, name: String) {
        self.scopes.push(Scope::new(name));
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn return_run_class(&self, mut new_run_class: RunClass) {
        new_run_class = self.branch_run_class.merge(new_run_class);
        if let Some(sym) = self.in_function {
            let mut b = self.syms[sym.0 as usize].borrow_mut();
            if let SymDecl::UserFunction(_, ref mut run_class) = b.decl {
                *run_class = run_class.merge(new_run_class);
            }
        }
    }

    pub fn function_definition(&self, name: SymRef) -> Option<(Rc<FunctionDefinition>, RunClass)> {
        if let SymDecl::UserFunction(ref fd, ref run_class) = &self.sym(name).decl {
            Some((fd.clone(), *run_class))
        } else {
            None
        }
    }

    fn merge_run_class(&self, sym: SymRef, mut new_run_class: RunClass) -> RunClass {
        if sym.0 <= self.branch_declaration.0 {
            new_run_class = self.branch_run_class.merge(new_run_class);
        }
        let mut b = self.syms[sym.0 as usize].borrow_mut();
        let mut old_run_class = new_run_class;
        if let SymDecl::Local(_, _, ref mut run_class) = b.decl {
            old_run_class = *run_class;
            new_run_class = old_run_class.merge(new_run_class);
            *run_class = new_run_class;
        }
        if old_run_class != RunClass::Unknown && old_run_class != new_run_class {
            self.run_class_changed.set(true);
        }
        new_run_class
    }
}

/// A declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    FunctionPrototype(FunctionPrototype),
    StructDefinition(SymRef),
    InitDeclaratorList(InitDeclaratorList),
    Precision(PrecisionQualifier, TypeSpecifier),
    Block(Block),
    Global(TypeQualifier, Vec<Identifier>),
}

/// A general purpose block, containing fields and possibly a list of declared identifiers. Semantic
/// is given with the storage qualifier.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub qualifier: TypeQualifier,
    pub name: Identifier,
    pub fields: Vec<StructFieldSpecifier>,
    pub identifier: Option<ArrayedIdentifier>,
}

/// Function identifier.
#[derive(Clone, Debug, PartialEq)]
pub enum FunIdentifier {
    Identifier(SymRef),
    Constructor(Type),
}

/// Function prototype.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionPrototype {
    pub ty: Type,
    pub name: Identifier,
    pub parameters: Vec<FunctionParameterDeclaration>,
}

impl FunctionPrototype {
    pub fn has_parameter(&self, sym: SymRef) -> bool {
        for param in &self.parameters {
            match param {
                FunctionParameterDeclaration::Named(_, ref d) => {
                    if d.sym == sym {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

/// Function parameter declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionParameterDeclaration {
    Named(Option<ParameterQualifier>, FunctionParameterDeclarator),
    Unnamed(Option<ParameterQualifier>, TypeSpecifier),
}

/// Function parameter declarator.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParameterDeclarator {
    pub ty: Type,
    pub name: Identifier,
    pub sym: SymRef,
}

/// Init declarator list.
#[derive(Clone, Debug, PartialEq)]
pub struct InitDeclaratorList {
    // XXX it feels like separating out the type and the names is better than
    // head and tail
    // Also, it might be nice to separate out type definitions from name definitions
    pub head: SingleDeclaration,
    pub tail: Vec<SingleDeclarationNoType>,
}

/// Type qualifier.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeQualifier {
    pub qualifiers: NonEmpty<TypeQualifierSpec>,
}

fn lift_type_qualifier_for_declaration(
    _state: &mut State,
    q: &Option<syntax::TypeQualifier>,
) -> Option<TypeQualifier> {
    q.as_ref().and_then(|x| {
        NonEmpty::from_non_empty_iter(x.qualifiers.0.iter().flat_map(|x| match x {
            syntax::TypeQualifierSpec::Precision(_) => None,
            syntax::TypeQualifierSpec::Interpolation(_) => None,
            syntax::TypeQualifierSpec::Invariant => Some(TypeQualifierSpec::Invariant),
            syntax::TypeQualifierSpec::Layout(l) => Some(TypeQualifierSpec::Layout(l.clone())),
            syntax::TypeQualifierSpec::Precise => Some(TypeQualifierSpec::Precise),
            syntax::TypeQualifierSpec::Storage(_) => None,
        }))
        .map(|x| TypeQualifier { qualifiers: x })
    })
}

fn lift_type_qualifier_for_parameter(
    _state: &mut State,
    q: &Option<syntax::TypeQualifier>,
) -> Option<ParameterQualifier> {
    let mut qp: Option<ParameterQualifier> = None;
    if let Some(q) = q {
        for x in &q.qualifiers.0 {
            match (&qp, x) {
                (None, syntax::TypeQualifierSpec::Storage(s)) => match s {
                    syntax::StorageQualifier::Const => qp = Some(ParameterQualifier::Const),
                    syntax::StorageQualifier::In => qp = Some(ParameterQualifier::In),
                    syntax::StorageQualifier::Out => qp = Some(ParameterQualifier::Out),
                    syntax::StorageQualifier::InOut => qp = Some(ParameterQualifier::InOut),
                    _ => panic!("Bad storage qualifier for parameter"),
                },
                (_, syntax::TypeQualifierSpec::Precision(_)) => {}
                _ => panic!("Bad parameter qualifier {:?}", x),
            }
        }
    }
    qp
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParameterQualifier {
    Const,
    In,
    InOut,
    Out,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryQualifier {
    Coherent,
    Volatile,
    Restrict,
    ReadOnly,
    WriteOnly,
}

/// Type qualifier spec.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeQualifierSpec {
    Layout(syntax::LayoutQualifier),
    Invariant,
    Parameter(ParameterQualifier),
    Memory(MemoryQualifier),
    Precise,
}

/// Single declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct SingleDeclaration {
    pub ty: Type,
    pub ty_def: Option<SymRef>,
    pub qualifier: Option<TypeQualifier>,
    pub name: SymRef,
    pub initializer: Option<Initializer>,
}

/// A single declaration with implicit, already-defined type.
#[derive(Clone, Debug, PartialEq)]
pub struct SingleDeclarationNoType {
    pub ident: ArrayedIdentifier,
    pub initializer: Option<Initializer>,
}

/// Initializer.
#[derive(Clone, Debug, PartialEq)]
pub enum Initializer {
    Simple(Box<Expr>),
    List(NonEmpty<Initializer>),
}

impl From<Expr> for Initializer {
    fn from(e: Expr) -> Self {
        Initializer::Simple(Box::new(e))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldSet {
    Rgba,
    Xyzw,
    Stpq,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwizzleSelector {
    pub field_set: FieldSet,
    pub components: Vec<i8>,
}

impl SwizzleSelector {
    fn parse(s: &str) -> Self {
        let mut components = Vec::new();
        let mut field_set = Vec::new();

        for c in s.chars() {
            match c {
                'r' => {
                    components.push(0);
                    field_set.push(FieldSet::Rgba);
                }
                'x' => {
                    components.push(0);
                    field_set.push(FieldSet::Xyzw);
                }
                's' => {
                    components.push(0);
                    field_set.push(FieldSet::Stpq);
                }

                'g' => {
                    components.push(1);
                    field_set.push(FieldSet::Rgba);
                }
                'y' => {
                    components.push(1);
                    field_set.push(FieldSet::Xyzw);
                }
                't' => {
                    components.push(1);
                    field_set.push(FieldSet::Stpq);
                }

                'b' => {
                    components.push(2);
                    field_set.push(FieldSet::Rgba);
                }
                'z' => {
                    components.push(2);
                    field_set.push(FieldSet::Xyzw);
                }
                'p' => {
                    components.push(2);
                    field_set.push(FieldSet::Stpq);
                }

                'a' => {
                    components.push(3);
                    field_set.push(FieldSet::Rgba);
                }
                'w' => {
                    components.push(3);
                    field_set.push(FieldSet::Xyzw);
                }
                'q' => {
                    components.push(3);
                    field_set.push(FieldSet::Stpq);
                }
                _ => panic!("bad selector"),
            }
        }

        let first = &field_set[0];
        assert!(field_set.iter().all(|item| item == first));
        assert!(components.len() <= 4);
        SwizzleSelector {
            field_set: first.clone(),
            components,
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        let fs = match self.field_set {
            FieldSet::Rgba => ['r', 'g', 'b', 'a'],
            FieldSet::Xyzw => ['x', 'y', 'z', 'w'],
            FieldSet::Stpq => ['s', 't', 'p', 'q'],
        };
        for i in &self.components {
            s.push(fs[*i as usize])
        }
        s
    }
}

/// The most general form of an expression. As you can see if you read the variant list, in GLSL, an
/// assignment is an expression. This is a bit silly but think of an assignment as a statement first
/// then an expression which evaluates to what the statement “returns”.
///
/// An expression is either an assignment or a list (comma) of assignments.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// A variable expression, using an identifier.
    Variable(SymRef),
    /// Integral constant expression.
    IntConst(i32),
    /// Unsigned integral constant expression.
    UIntConst(u32),
    /// Boolean constant expression.
    BoolConst(bool),
    /// Single precision floating expression.
    FloatConst(f32),
    /// Double precision floating expression.
    DoubleConst(f64),
    /// A unary expression, gathering a single expression and a unary operator.
    Unary(UnaryOp, Box<Expr>),
    /// A binary expression, gathering two expressions and a binary operator.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// A ternary conditional expression, gathering three expressions.
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    /// An assignment is also an expression. Gathers an expression that defines what to assign to, an
    /// assignment operator and the value to associate with.
    Assignment(Box<Expr>, AssignmentOp, Box<Expr>),
    /// Add an array specifier to an expression.
    Bracket(Box<Expr>, Box<Expr>),
    /// A functional call. It has a function identifier and a list of expressions (arguments).
    FunCall(FunIdentifier, Vec<Expr>),
    /// An expression associated with a field selection (struct).
    Dot(Box<Expr>, Identifier),
    /// An expression associated with a component selection
    SwizzleSelector(Box<Expr>, SwizzleSelector),
    /// Post-incrementation of an expression.
    PostInc(Box<Expr>),
    /// Post-decrementation of an expression.
    PostDec(Box<Expr>),
    /// An expression that contains several, separated with comma.
    Comma(Box<Expr>, Box<Expr>),
    /// A temporary condition variable
    Cond(usize, Box<Expr>),
    CondMask,
}

/*
impl From<i32> for Expr {
    fn from(x: i32) -> Expr {
        ExprKind::IntConst(x)
    }
}

impl From<u32> for Expr {
    fn from(x: u32) -> Expr {
        Expr::UIntConst(x)
    }
}

impl From<bool> for Expr {
    fn from(x: bool) -> Expr {
        Expr::BoolConst(x)
    }
}

impl From<f32> for Expr {
    fn from(x: f32) -> Expr {
        Expr::FloatConst(x)
    }
}

impl From<f64> for Expr {
    fn from(x: f64) -> Expr {
        Expr::DoubleConst(x)
    }
}
*/
/// Starting rule.
#[derive(Clone, Debug, PartialEq)]
pub struct TranslationUnit(pub NonEmpty<ExternalDeclaration>);

impl TranslationUnit {
    /// Construct a translation unit from an iterator.
    ///
    /// # Errors
    ///
    /// `None` if the iterator yields no value.
    pub fn from_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = ExternalDeclaration>,
    {
        NonEmpty::from_non_empty_iter(iter).map(TranslationUnit)
    }
}

impl Deref for TranslationUnit {
    type Target = NonEmpty<ExternalDeclaration>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TranslationUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for TranslationUnit {
    type IntoIter = <NonEmpty<ExternalDeclaration> as IntoIterator>::IntoIter;
    type Item = ExternalDeclaration;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a TranslationUnit {
    type IntoIter = <&'a NonEmpty<ExternalDeclaration> as IntoIterator>::IntoIter;
    type Item = &'a ExternalDeclaration;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut TranslationUnit {
    type IntoIter = <&'a mut NonEmpty<ExternalDeclaration> as IntoIterator>::IntoIter;
    type Item = &'a mut ExternalDeclaration;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

/// External declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum ExternalDeclaration {
    Preprocessor(syntax::Preprocessor),
    FunctionDefinition(Rc<FunctionDefinition>),
    Declaration(Declaration),
}

/// Function definition.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    pub prototype: FunctionPrototype,
    pub body: CompoundStatement,
    pub globals: Vec<SymRef>,
    pub texel_fetches: HashMap<(SymRef, SymRef), TexelFetchOffsets>,
}

/// Compound statement (with no new scope).
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundStatement {
    pub statement_list: Vec<Statement>,
}

impl CompoundStatement {
    pub fn new() -> Self {
        CompoundStatement {
            statement_list: Vec::new(),
        }
    }
}

impl FromIterator<Statement> for CompoundStatement {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Statement>,
    {
        CompoundStatement {
            statement_list: iter.into_iter().collect(),
        }
    }
}

/// Statement.
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Compound(Box<CompoundStatement>),
    Simple(Box<SimpleStatement>),
}

/// Simple statement.
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleStatement {
    Declaration(Declaration),
    Expression(ExprStatement),
    Selection(SelectionStatement),
    Switch(SwitchStatement),
    Iteration(IterationStatement),
    Jump(JumpStatement),
}

impl SimpleStatement {
    /// Create a new expression statement.
    pub fn new_expr<E>(expr: E) -> Self
    where
        E: Into<Expr>,
    {
        SimpleStatement::Expression(Some(expr.into()))
    }

    /// Create a new selection statement (if / else).
    pub fn new_if_else<If, True, False>(ife: If, truee: True, falsee: False) -> Self
    where
        If: Into<Expr>,
        True: Into<Statement>,
        False: Into<Statement>,
    {
        SimpleStatement::Selection(SelectionStatement {
            cond: Box::new(ife.into()),
            body: Box::new(truee.into()),
            else_stmt: Some(Box::new(falsee.into())),
        })
    }

    /// Create a new while statement.
    pub fn new_while<C, S>(cond: C, body: S) -> Self
    where
        C: Into<Condition>,
        S: Into<Statement>,
    {
        SimpleStatement::Iteration(IterationStatement::While(
            cond.into(),
            Box::new(body.into()),
        ))
    }

    /// Create a new do-while statement.
    pub fn new_do_while<C, S>(body: S, cond: C) -> Self
    where
        S: Into<Statement>,
        C: Into<Expr>,
    {
        SimpleStatement::Iteration(IterationStatement::DoWhile(
            Box::new(body.into()),
            Box::new(cond.into()),
        ))
    }
}

/// Expression statement.
pub type ExprStatement = Option<Expr>;

/// Selection statement.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionStatement {
    pub cond: Box<Expr>,
    pub body: Box<Statement>,
    // the else branch
    pub else_stmt: Option<Box<Statement>>,
}

/// Condition.
#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Expr(Box<Expr>),
}

impl From<Expr> for Condition {
    fn from(expr: Expr) -> Self {
        Condition::Expr(Box::new(expr))
    }
}

/// Switch statement.
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchStatement {
    pub head: Box<Expr>,
    pub cases: Vec<Case>,
}

/// Case label statement.
#[derive(Clone, Debug, PartialEq)]
pub enum CaseLabel {
    Case(Box<Expr>),
    Def,
}

/// An individual case
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub label: CaseLabel,
    pub stmts: Vec<Statement>,
}

/// Iteration statement.
#[derive(Clone, Debug, PartialEq)]
pub enum IterationStatement {
    While(Condition, Box<Statement>),
    DoWhile(Box<Statement>, Box<Expr>),
    For(ForInitStatement, ForRestStatement, Box<Statement>),
}

/// For init statement.
#[derive(Clone, Debug, PartialEq)]
pub enum ForInitStatement {
    Expression(Option<Expr>),
    Declaration(Box<Declaration>),
}

/// For init statement.
#[derive(Clone, Debug, PartialEq)]
pub struct ForRestStatement {
    pub condition: Option<Condition>,
    pub post_expr: Option<Box<Expr>>,
}

/// Jump statement.
#[derive(Clone, Debug, PartialEq)]
pub enum JumpStatement {
    Continue,
    Break,
    Return(Option<Box<Expr>>),
    Discard,
}

trait NonEmptyExt<T> {
    fn map<U, F: FnMut(&mut State, &T) -> U>(&self, s: &mut State, f: F) -> NonEmpty<U>;
    fn new(x: T) -> NonEmpty<T>;
}

impl<T> NonEmptyExt<T> for NonEmpty<T> {
    fn map<U, F: FnMut(&mut State, &T) -> U>(&self, s: &mut State, mut f: F) -> NonEmpty<U> {
        NonEmpty::from_non_empty_iter(self.into_iter().map(|x| f(s, &x))).unwrap()
    }
    fn new(x: T) -> NonEmpty<T> {
        NonEmpty::from_non_empty_iter(vec![x].into_iter()).unwrap()
    }
}

fn translate_initializater(state: &mut State, i: &syntax::Initializer) -> Initializer {
    match i {
        syntax::Initializer::Simple(i) => {
            Initializer::Simple(Box::new(translate_expression(state, i)))
        }
        _ => panic!(),
    }
}

fn translate_struct_declaration(state: &mut State, d: &syntax::SingleDeclaration) -> Declaration {
    let ty = d.ty.clone();
    let ty_def = match &ty.ty.ty {
        TypeSpecifierNonArray::Struct(s) => {
            let decl = SymDecl::Struct(lift(state, s));
            Some(state.declare(s.name.as_ref().unwrap().as_str(), decl))
        }
        _ => None,
    };

    let ty_def = ty_def.expect("Must be type definition");

    Declaration::StructDefinition(ty_def)
}

fn get_expr_index(e: &syntax::Expr) -> i32 {
    match e {
        syntax::Expr::IntConst(i) => *i,
        syntax::Expr::UIntConst(u) => *u as i32,
        syntax::Expr::FloatConst(f) => *f as i32,
        syntax::Expr::DoubleConst(f) => *f as i32,
        _ => panic!(),
    }
}

fn translate_variable_declaration(
    state: &mut State,
    d: &syntax::InitDeclaratorList,
    default_run_class: RunClass,
) -> Declaration {
    let mut ty = d.head.ty.clone();
    ty.ty.array_specifier = d.head.array_specifier.clone();
    let ty_def = match &ty.ty.ty {
        TypeSpecifierNonArray::Struct(s) => {
            let decl = SymDecl::Struct(lift(state, s));
            Some(state.declare(s.name.as_ref().unwrap().as_str(), decl))
        }
        _ => None,
    };

    let mut ty: Type = lift(state, &d.head.ty);
    if let Some(array) = &d.head.array_specifier {
        ty.array_sizes = Some(Box::new(lift(state, array)))
    }

    let (sym, decl) = match d.head.name.as_ref() {
        Some(name) => {
            let mut storage = StorageClass::None;
            let mut interpolation = None;
            for qual in d
                .head
                .ty
                .qualifier
                .iter()
                .flat_map(|x| x.qualifiers.0.iter())
            {
                match qual {
                    syntax::TypeQualifierSpec::Storage(s) => match (&storage, s) {
                        (StorageClass::FragColor(..), syntax::StorageQualifier::Out) => {}
                        (StorageClass::Sampler(..), syntax::StorageQualifier::Uniform) => {}
                        (StorageClass::None, syntax::StorageQualifier::Out) => {
                            storage = StorageClass::Out;
                        }
                        (StorageClass::None, syntax::StorageQualifier::In) => {
                            storage = StorageClass::In;
                        }
                        (StorageClass::None, syntax::StorageQualifier::Uniform) => {
                            if ty.kind.is_sampler() {
                                storage = StorageClass::Sampler(SamplerFormat::Unknown);
                            } else {
                                storage = StorageClass::Uniform;
                            }
                        }
                        (StorageClass::None, syntax::StorageQualifier::Const) => {
                            storage = StorageClass::Const;
                        }
                        _ => panic!("bad storage {:?}", (storage, s)),
                    },
                    syntax::TypeQualifierSpec::Interpolation(i) => match (&interpolation, i) {
                        (None, i) => interpolation = Some(i.clone()),
                        _ => panic!("multiple interpolation"),
                    },
                    syntax::TypeQualifierSpec::Layout(l) => {
                        let mut loc = -1;
                        let mut index = -1;
                        for id in &l.ids {
                            match id {
                                syntax::LayoutQualifierSpec::Identifier(ref key, None) => {
                                    match key.as_str() {
                                        "rgba8" => {
                                            storage = StorageClass::Sampler(SamplerFormat::RGBA8);
                                        }
                                        "rgba32f" => {
                                            storage = StorageClass::Sampler(SamplerFormat::RGBA32F);
                                        }
                                        "rgba32i" => {
                                            storage = StorageClass::Sampler(SamplerFormat::RGBA32I);
                                        }
                                        "r8" => {
                                            storage = StorageClass::Sampler(SamplerFormat::R8);
                                        }
                                        _ => {}
                                    }
                                }
                                syntax::LayoutQualifierSpec::Identifier(ref key, Some(ref e)) => {
                                    match key.as_str() {
                                        "location" => {
                                            loc = get_expr_index(e);
                                        }
                                        "index" => {
                                            index = get_expr_index(e);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        if index >= 0 {
                            assert!(loc == 0);
                            assert!(index <= 1);
                            assert!(storage == StorageClass::None);
                            storage = StorageClass::FragColor(index);
                        }
                    }
                    _ => {}
                }
            }
            let decl = if state.in_function.is_some() {
                let run_class = match storage {
                    StorageClass::Const => RunClass::Scalar,
                    StorageClass::None => default_run_class,
                    _ => panic!("bad local storage {:?}", storage),
                };
                SymDecl::Local(storage, ty.clone(), run_class)
            } else {
                let run_class = match storage {
                    StorageClass::Const | StorageClass::Uniform | StorageClass::Sampler(..) => {
                        RunClass::Scalar
                    }
                    StorageClass::In | StorageClass::Out | StorageClass::FragColor(..)
                        if interpolation == Some(syntax::InterpolationQualifier::Flat) =>
                    {
                        RunClass::Scalar
                    }
                    _ => RunClass::Vector,
                };
                SymDecl::Global(storage, interpolation, ty.clone(), run_class)
            };
            (state.declare(name.as_str(), decl.clone()), decl)
        }
        None => panic!(),
    };

    let head = SingleDeclaration {
        qualifier: lift_type_qualifier_for_declaration(state, &d.head.ty.qualifier),
        name: sym,
        ty,
        ty_def,
        initializer: d
            .head
            .initializer
            .as_ref()
            .map(|x| translate_initializater(state, x)),
    };

    let tail = d
        .tail
        .iter()
        .map(|d| {
            if let Some(_array) = &d.ident.array_spec {
                panic!("unhandled array")
            }
            state.declare(d.ident.ident.as_str(), decl.clone());
            SingleDeclarationNoType {
                ident: d.ident.clone(),
                initializer: d
                    .initializer
                    .as_ref()
                    .map(|x| translate_initializater(state, x)),
            }
        })
        .collect();
    Declaration::InitDeclaratorList(InitDeclaratorList { head, tail })
}

fn translate_init_declarator_list(
    state: &mut State,
    l: &syntax::InitDeclaratorList,
    default_run_class: RunClass,
) -> Declaration {
    match &l.head.name {
        Some(_name) => translate_variable_declaration(state, l, default_run_class),
        None => translate_struct_declaration(state, &l.head),
    }
}

fn translate_declaration(
    state: &mut State,
    d: &syntax::Declaration,
    default_run_class: RunClass,
) -> Declaration {
    match d {
        syntax::Declaration::Block(_) => panic!(), //Declaration::Block(..),
        syntax::Declaration::FunctionPrototype(p) => {
            Declaration::FunctionPrototype(translate_function_prototype(state, p))
        }
        syntax::Declaration::Global(_ty, _ids) => {
            panic!();
            // glsl non-es supports requalifying variables
            // we don't right now
            //Declaration::Global(..)
        }
        syntax::Declaration::InitDeclaratorList(dl) => {
            translate_init_declarator_list(state, dl, default_run_class)
        }
        syntax::Declaration::Precision(p, ts) => Declaration::Precision(p.clone(), ts.clone()),
    }
}

fn is_vector(ty: &Type) -> bool {
    match ty.kind {
        TypeKind::Vec2
        | TypeKind::Vec3
        | TypeKind::Vec4
        | TypeKind::IVec2
        | TypeKind::IVec3
        | TypeKind::IVec4 => ty.array_sizes == None,
        _ => false,
    }
}

fn index_matrix(ty: &Type) -> Option<TypeKind> {
    use TypeKind::*;
    if ty.array_sizes != None {
        return None;
    }
    Some(match ty.kind {
        Mat2 => Vec2,
        Mat3 => Vec3,
        Mat4 => Vec4,
        Mat23 => Vec3,
        Mat24 => Vec4,
        Mat32 => Vec2,
        Mat34 => Vec4,
        Mat42 => Vec2,
        Mat43 => Vec3,
        DMat2 => DVec2,
        DMat3 => DVec3,
        DMat4 => DVec4,
        DMat23 => DVec3,
        DMat24 => DVec4,
        DMat32 => DVec2,
        DMat34 => DVec4,
        DMat42 => DVec2,
        DMat43 => DVec3,
        _ => return None,
    })
}

fn is_ivec(ty: &Type) -> bool {
    match ty.kind {
        TypeKind::IVec2 | TypeKind::IVec3 | TypeKind::IVec4 => ty.array_sizes == None,
        _ => false,
    }
}

fn compatible_type(lhs: &Type, rhs: &Type) -> bool {
    // XXX: use an underlying type helper
    if lhs == &Type::new(TypeKind::Double) && rhs == &Type::new(TypeKind::Float) {
        true
    } else if rhs == &Type::new(TypeKind::Double) && lhs == &Type::new(TypeKind::Float) {
        true
    } else if rhs == &Type::new(TypeKind::Int) &&
        (lhs == &Type::new(TypeKind::Float) || lhs == &Type::new(TypeKind::Double))
    {
        true
    } else if (rhs == &Type::new(TypeKind::Float) || rhs == &Type::new(TypeKind::Double)) &&
        lhs == &Type::new(TypeKind::Int)
    {
        true
    } else if (rhs == &Type::new(TypeKind::Vec2) || rhs == &Type::new(TypeKind::DVec2)) &&
        lhs == &Type::new(TypeKind::IVec2)
    {
        true
    } else if rhs == &Type::new(TypeKind::IVec2) &&
        (lhs == &Type::new(TypeKind::Vec2) || lhs == &Type::new(TypeKind::DVec2))
    {
        true
    } else {
        lhs.kind == rhs.kind && lhs.array_sizes == rhs.array_sizes
    }
}

fn promoted_type(lhs: &Type, rhs: &Type) -> Type {
    if lhs == &Type::new(TypeKind::Double) && rhs == &Type::new(TypeKind::Float) {
        Type::new(TypeKind::Double)
    } else if lhs == &Type::new(TypeKind::Float) && rhs == &Type::new(TypeKind::Double) {
        Type::new(TypeKind::Double)
    } else if lhs == &Type::new(TypeKind::Int) && rhs == &Type::new(TypeKind::Double) {
        Type::new(TypeKind::Double)
    } else if is_vector(&lhs) &&
        (rhs == &Type::new(TypeKind::Float) ||
         rhs == &Type::new(TypeKind::Double) ||
         rhs == &Type::new(TypeKind::Int))
    {
        // scalars promote to vectors
        lhs.clone()
    } else if is_vector(&rhs) &&
        (lhs == &Type::new(TypeKind::Float) ||
         lhs == &Type::new(TypeKind::Double) ||
         lhs == &Type::new(TypeKind::Int))
    {
        // scalars promote to vectors
        rhs.clone()
    } else if lhs == rhs {
        lhs.clone()
    } else if lhs.kind == rhs.kind {
        if lhs.array_sizes == rhs.array_sizes {
            // XXX: we need to be able to query the default precision here
            match (&lhs.precision, &rhs.precision) {
                (Some(PrecisionQualifier::High), _) => lhs.clone(),
                (_, Some(PrecisionQualifier::High)) => rhs.clone(),
                (None, _) => lhs.clone(),
                (_, None) => rhs.clone(),
                _ => panic!("precision mismatch {:?} {:?}", lhs.precision, rhs.precision),
            }
        } else {
            panic!("array size mismatch")
        }
    } else {
        assert_eq!(lhs, rhs);
        lhs.clone()
    }
}

pub fn is_output(expr: &Expr, state: &State) -> Option<SymRef> {
    match &expr.kind {
        ExprKind::Variable(i) => match state.sym(*i).decl {
            SymDecl::Global(storage, ..) => match storage {
                StorageClass::Out => return Some(*i),
                _ => {}
            },
            SymDecl::Local(..) => {}
            _ => panic!("should be variable"),
        },
        ExprKind::SwizzleSelector(e, ..) => {
            return is_output(e, state);
        }
        ExprKind::Bracket(e, ..) => {
            return is_output(e, state);
        }
        ExprKind::Dot(e, ..) => {
            return is_output(e, state);
        }
        _ => {}
    };
    None
}

pub fn get_texel_fetch_offset(
    state: &State,
    sampler_expr: &Expr,
    uv_expr: &Expr,
    offset_expr: &Expr,
) -> Option<(SymRef, SymRef, i32, i32)> {
    if let ExprKind::Variable(ref sampler) = &sampler_expr.kind {
        //if let ExprKind::Binary(BinaryOp::Add, ref lhs, ref rhs) = &uv_expr.kind {
        if let ExprKind::Variable(ref base) = &uv_expr.kind {
            if let ExprKind::FunCall(ref fun, ref args) = &offset_expr.kind {
                if let FunIdentifier::Identifier(ref offset) = fun {
                    if state.sym(*offset).name == "ivec2" {
                        if let ExprKind::IntConst(ref x) = &args[0].kind {
                            if let ExprKind::IntConst(ref y) = &args[1].kind {
                                return Some((*sampler, *base, *x, *y));
                            }
                        }
                    }
                }
            }
        }
        //}
    }
    None
}

fn make_const(t: TypeKind, v: i32) -> Expr {
    Expr {
        kind: match t {
            TypeKind::Int => ExprKind::IntConst(v as _),
            TypeKind::UInt => ExprKind::UIntConst(v as _),
            TypeKind::Bool => ExprKind::BoolConst(v != 0),
            TypeKind::Float => ExprKind::FloatConst(v as _),
            TypeKind::Double => ExprKind::DoubleConst(v as _),
            _ => panic!("bad constant type"),
        },
        ty: Type::new(t),
    }
}

// Any parameters needing to convert to bool should just compare via != 0.
// This ensures they get the proper all-1s pattern for C++ OpenCL vectors.
fn force_params_to_bool(_state: &mut State, params: &mut Vec<Expr>) {
    for e in params {
        if !e.ty.kind.is_bool() {
            let k = e.ty.kind;
            *e = Expr {
                kind: ExprKind::Binary(
                    BinaryOp::NonEqual,
                    Box::new(e.clone()),
                    Box::new(make_const(k.to_scalar(), 0)),
                ),
                ty: Type::new(k.to_bool()),
            };
        }
    }
}

// Transform bool params to int, then mask off the low bit so they become 0 or 1.
// C++ OpenCL vectors represent bool as all-1s patterns, which will erroneously
// convert to -1 otherwise.
fn force_params_from_bool(state: &mut State, params: &mut Vec<Expr>) {
    for e in params {
        if e.ty.kind.is_bool() {
            let k = e.ty.kind.to_int();
            let sym = state.lookup(k.glsl_primitive_type_name().unwrap()).unwrap();
            *e = Expr {
                kind: ExprKind::Binary(
                    BinaryOp::BitAnd,
                    Box::new(Expr {
                        kind: ExprKind::FunCall(
                            FunIdentifier::Identifier(sym),
                            vec![e.clone()],
                        ),
                        ty: Type::new(k),
                    }),
                    Box::new(make_const(TypeKind::Int, 1)),
                ),
                ty: Type::new(k),
            };
        }
    }
}

fn translate_expression(state: &mut State, e: &syntax::Expr) -> Expr {
    match e {
        syntax::Expr::Variable(i) => {
            let sym = match state.lookup(i.as_str()) {
                Some(sym) => sym,
                None => panic!("missing declaration {}", i.as_str()),
            };
            let ty = match &state.sym(sym).decl {
                SymDecl::Global(_, _, ty, _) => {
                    let mut globals = state.used_globals.borrow_mut();
                    if !globals.contains(&sym) {
                        globals.push(sym);
                    }
                    ty.clone()
                }
                SymDecl::Local(_, ty, _) => ty.clone(),
                _ => panic!("bad variable type"),
            };
            Expr {
                kind: ExprKind::Variable(sym),
                ty,
            }
        }
        syntax::Expr::Assignment(lhs, op, rhs) => {
            let lhs = Box::new(translate_expression(state, lhs));
            let rhs = Box::new(translate_expression(state, rhs));
            let ty = if op == &AssignmentOp::Mult {
                if lhs.ty.kind == TypeKind::Vec4 && rhs.ty.kind == TypeKind::Float {
                    lhs.ty.clone()
                } else {
                    promoted_type(&lhs.ty, &rhs.ty)
                }
            } else {
                promoted_type(&lhs.ty, &rhs.ty)
            };
            if let Some(global) = is_output(&lhs, state) {
                let mut globals = state.modified_globals.borrow_mut();
                if !globals.contains(&global) {
                    globals.push(global);
                }
            }
            Expr {
                kind: ExprKind::Assignment(lhs, op.clone(), rhs),
                ty,
            }
        }
        syntax::Expr::Binary(op, lhs, rhs) => {
            let lhs = Box::new(translate_expression(state, lhs));
            let rhs = Box::new(translate_expression(state, rhs));
            let ty = if op == &BinaryOp::Mult {
                if lhs.ty.kind == TypeKind::Mat3 && rhs.ty.kind == TypeKind::Vec3 {
                    rhs.ty.clone()
                } else if lhs.ty.kind == TypeKind::Mat4 && rhs.ty.kind == TypeKind::Vec4 {
                    rhs.ty.clone()
                } else if lhs.ty.kind == TypeKind::Mat2 && rhs.ty.kind == TypeKind::Vec2 {
                    rhs.ty.clone()
                } else if lhs.ty.kind == TypeKind::Mat2 && rhs.ty.kind == TypeKind::Float {
                    lhs.ty.clone()
                } else {
                    promoted_type(&lhs.ty, &rhs.ty)
                }
            } else {
                promoted_type(&lhs.ty, &rhs.ty)
            };

            // comparison operators have a bool result
            let ty = match op {
                BinaryOp::Equal | BinaryOp::GT | BinaryOp::GTE | BinaryOp::LT | BinaryOp::LTE => {
                    Type::new(TypeKind::Bool)
                }
                _ => ty,
            };

            Expr {
                kind: ExprKind::Binary(op.clone(), lhs, rhs),
                ty,
            }
        }
        syntax::Expr::Unary(op, e) => {
            let e = Box::new(translate_expression(state, e));
            let ty = e.ty.clone();
            Expr {
                kind: ExprKind::Unary(op.clone(), e),
                ty,
            }
        }
        syntax::Expr::BoolConst(b) => Expr {
            kind: ExprKind::BoolConst(*b),
            ty: Type::new(TypeKind::Bool),
        },
        syntax::Expr::Comma(lhs, rhs) => {
            let lhs = Box::new(translate_expression(state, lhs));
            let rhs = Box::new(translate_expression(state, rhs));
            assert_eq!(lhs.ty, rhs.ty);
            let ty = lhs.ty.clone();
            Expr {
                kind: ExprKind::Comma(lhs, rhs),
                ty,
            }
        }
        syntax::Expr::DoubleConst(d) => Expr {
            kind: ExprKind::DoubleConst(*d),
            ty: Type::new(TypeKind::Double),
        },
        syntax::Expr::FloatConst(f) => Expr {
            kind: ExprKind::FloatConst(*f),
            ty: Type::new(TypeKind::Float),
        },
        syntax::Expr::FunCall(fun, params) => {
            let ret_ty: Type;
            let mut params: Vec<Expr> = params
                .iter()
                .map(|x| translate_expression(state, x))
                .collect();
            Expr {
                kind: ExprKind::FunCall(
                    match fun {
                        syntax::FunIdentifier::Identifier(i) => {
                            let name = i.as_str();
                            if name == "texelFetchOffset" && params.len() >= 4 {
                                if let Some((sampler, base, x, y)) = get_texel_fetch_offset(
                                    state, &params[0], &params[1], &params[3],
                                ) {
                                    if let Some(offsets) =
                                        state.texel_fetches.get_mut(&(sampler, base))
                                    {
                                        offsets.add_offset(x, y);
                                    } else {
                                        state
                                            .texel_fetches
                                            .insert((sampler, base), TexelFetchOffsets::new(x, y));
                                    }
                                }
                            }
                            let sym = match state.lookup(name) {
                                Some(s) => s,
                                None => panic!("missing symbol {}", name),
                            };
                            // Force any boolean basic type constructors to generate correct
                            // bit patterns.
                            if let Some(t) = TypeKind::from_glsl_primitive_type_name(name) {
                                if t.is_bool() {
                                    force_params_to_bool(state, &mut params);
                                } else {
                                    force_params_from_bool(state, &mut params);
                                }
                            }
                            match &state.sym(sym).decl {
                                SymDecl::NativeFunction(fn_ty, _) => {
                                    let mut ret = None;
                                    for sig in &fn_ty.signatures {
                                        let mut matching = true;
                                        for (e, p) in params.iter().zip(sig.params.iter()) {
                                            if !compatible_type(&e.ty, p) {
                                                matching = false;
                                                break;
                                            }
                                        }
                                        if matching {
                                            ret = Some(sig.ret.clone());
                                            break;
                                        }
                                    }
                                    ret_ty = match ret {
                                        Some(t) => t,
                                        None => {
                                            dbg!(&fn_ty.signatures);
                                            dbg!(params.iter().map(|p| p).collect::<Vec<_>>());
                                            panic!("no matching func {}", i.as_str())
                                        }
                                    };
                                }
                                SymDecl::UserFunction(fd, _) => {
                                    let mut globals = state.modified_globals.borrow_mut();
                                    for global in &fd.globals {
                                        if !globals.contains(global) {
                                            globals.push(*global);
                                        }
                                    }
                                    let mut matching = true;
                                    for (e, p) in params.iter().zip(fd.prototype.parameters.iter())
                                    {
                                        matching &= match p {
                                            FunctionParameterDeclaration::Named(q, d) => {
                                                match q {
                                                    Some(ParameterQualifier::InOut)
                                                    | Some(ParameterQualifier::Out) => {
                                                        if let Some(global) = is_output(e, state) {
                                                            if !globals.contains(&global) {
                                                                globals.push(global);
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                compatible_type(&e.ty, &d.ty)
                                            }
                                            FunctionParameterDeclaration::Unnamed(..) => panic!(),
                                        };
                                    }
                                    assert!(matching);
                                    ret_ty = fd.prototype.ty.clone();
                                }
                                SymDecl::Struct(_) => ret_ty = Type::new(TypeKind::Struct(sym)),
                                _ => panic!("can only call functions"),
                            };
                            FunIdentifier::Identifier(sym)
                        }
                        // array constructor
                        syntax::FunIdentifier::Expr(e) => {
                            let ty = match &**e {
                                syntax::Expr::Bracket(i, array) => {
                                    let kind = match &**i {
                                        syntax::Expr::Variable(i) => match i.as_str() {
                                            "vec4" => TypeKind::Vec4,
                                            "vec2" => TypeKind::Vec2,
                                            _ => panic!("unexpected type constructor {:?}", i),
                                        },
                                        _ => panic!(),
                                    };

                                    Type {
                                        kind,
                                        precision: None,
                                        array_sizes: Some(Box::new(lift(state, array))),
                                    }
                                }
                                _ => panic!(),
                            };
                            ret_ty = ty.clone();

                            FunIdentifier::Constructor(ty)
                        }
                    },
                    params,
                ),
                ty: ret_ty,
            }
        }
        syntax::Expr::IntConst(i) => Expr {
            kind: ExprKind::IntConst(*i),
            ty: Type::new(TypeKind::Int),
        },
        syntax::Expr::UIntConst(u) => Expr {
            kind: ExprKind::UIntConst(*u),
            ty: Type::new(TypeKind::UInt),
        },
        syntax::Expr::PostDec(e) => {
            let e = Box::new(translate_expression(state, e));
            let ty = e.ty.clone();
            Expr {
                kind: ExprKind::PostDec(e),
                ty,
            }
        }
        syntax::Expr::PostInc(e) => {
            let e = Box::new(translate_expression(state, e));
            let ty = e.ty.clone();
            Expr {
                kind: ExprKind::PostInc(e),
                ty,
            }
        }
        syntax::Expr::Ternary(cond, lhs, rhs) => {
            let cond = Box::new(translate_expression(state, cond));
            let lhs = Box::new(translate_expression(state, lhs));
            let rhs = Box::new(translate_expression(state, rhs));
            let ty = promoted_type(&lhs.ty, &rhs.ty);
            Expr {
                kind: ExprKind::Ternary(cond, lhs, rhs),
                ty,
            }
        }
        syntax::Expr::Dot(e, i) => {
            let e = Box::new(translate_expression(state, e));
            let ty = e.ty.clone();
            let ivec = is_ivec(&ty);
            if is_vector(&ty) {
                let ty = Type::new(match i.as_str().len() {
                    1 => {
                        if ivec {
                            TypeKind::Int
                        } else {
                            TypeKind::Float
                        }
                    }
                    2 => {
                        if ivec {
                            TypeKind::IVec2
                        } else {
                            TypeKind::Vec2
                        }
                    }
                    3 => {
                        if ivec {
                            TypeKind::IVec3
                        } else {
                            TypeKind::Vec3
                        }
                    }
                    4 => {
                        if ivec {
                            TypeKind::IVec4
                        } else {
                            TypeKind::Vec4
                        }
                    }
                    _ => panic!(),
                });

                let sel = SwizzleSelector::parse(i.as_str());

                Expr {
                    kind: ExprKind::SwizzleSelector(e, sel),
                    ty,
                }
            } else {
                match ty.kind {
                    TypeKind::Struct(s) => {
                        let sym = state.sym(s);
                        let fields = match &sym.decl {
                            SymDecl::Struct(fields) => fields,
                            _ => panic!("expected struct"),
                        };
                        let field = fields
                            .fields
                            .iter()
                            .find(|x| &x.name == i)
                            .expect("missing field");
                        Expr {
                            kind: ExprKind::Dot(e, i.clone()),
                            ty: field.ty.clone(),
                        }
                    }
                    _ => panic!("expected struct found {:#?} {:#?}", e, ty),
                }
            }
        }
        syntax::Expr::Bracket(e, specifier) => {
            let e = Box::new(translate_expression(state, e));
            let ty = if is_vector(&e.ty) {
                Type::new(TypeKind::Float)
            } else if let Some(ty) = index_matrix(&e.ty) {
                Type::new(ty)
            } else {
                let a = match &e.ty.array_sizes {
                    Some(a) => {
                        let mut a = *a.clone();
                        a.sizes.pop();
                        if a.sizes.len() == 0 {
                            None
                        } else {
                            Some(Box::new(a))
                        }
                    }
                    _ => panic!("{:#?}", e),
                };
                Type {
                    kind: e.ty.kind.clone(),
                    precision: e.ty.precision.clone(),
                    array_sizes: a,
                }
            };
            let indx = match specifier {
                ArraySpecifier::Unsized => panic!("need expression"),
                ArraySpecifier::ExplicitlySized(e) => translate_expression(state, e),
            };
            Expr {
                kind: ExprKind::Bracket(e, Box::new(indx)),
                ty,
            }
        }
    }
}

fn translate_switch(state: &mut State, s: &syntax::SwitchStatement) -> SwitchStatement {
    let mut cases = Vec::new();

    let mut case = None;
    for stmt in &s.body {
        match stmt {
            syntax::Statement::Simple(s) => match &**s {
                syntax::SimpleStatement::CaseLabel(label) => {
                    match case.take() {
                        Some(case) => cases.push(case),
                        _ => {}
                    }
                    case = Some(Case {
                        label: translate_case(state, &label),
                        stmts: Vec::new(),
                    })
                }
                _ => match case {
                    Some(ref mut case) => case.stmts.push(translate_statement(state, stmt)),
                    _ => panic!("switch must start with case"),
                },
            },
            _ => match case {
                Some(ref mut case) => case.stmts.push(translate_statement(state, stmt)),
                _ => panic!("switch must start with case"),
            },
        }
    }
    match case.take() {
        Some(case) => cases.push(case),
        _ => {}
    }
    SwitchStatement {
        head: Box::new(translate_expression(state, &s.head)),
        cases,
    }
}

fn translate_jump(state: &mut State, s: &syntax::JumpStatement) -> JumpStatement {
    match s {
        syntax::JumpStatement::Break => JumpStatement::Break,
        syntax::JumpStatement::Continue => JumpStatement::Continue,
        syntax::JumpStatement::Discard => JumpStatement::Discard,
        syntax::JumpStatement::Return(e) => {
            JumpStatement::Return(e.as_ref().map(|e| Box::new(translate_expression(state, e))))
        }
    }
}

fn translate_condition(state: &mut State, c: &syntax::Condition) -> Condition {
    match c {
        syntax::Condition::Expr(e) => Condition::Expr(Box::new(translate_expression(state, e))),
        _ => panic!(),
    }
}

fn translate_for_init(state: &mut State, s: &syntax::ForInitStatement) -> ForInitStatement {
    match s {
        syntax::ForInitStatement::Expression(e) => {
            ForInitStatement::Expression(e.as_ref().map(|e| translate_expression(state, e)))
        }
        syntax::ForInitStatement::Declaration(d) => ForInitStatement::Declaration(Box::new(
            translate_declaration(state, d, RunClass::Scalar),
        )),
    }
}

fn translate_for_rest(state: &mut State, s: &syntax::ForRestStatement) -> ForRestStatement {
    ForRestStatement {
        condition: s.condition.as_ref().map(|c| translate_condition(state, c)),
        post_expr: s
            .post_expr
            .as_ref()
            .map(|e| Box::new(translate_expression(state, e))),
    }
}

fn translate_iteration(state: &mut State, s: &syntax::IterationStatement) -> IterationStatement {
    match s {
        syntax::IterationStatement::While(cond, s) => IterationStatement::While(
            translate_condition(state, cond),
            Box::new(translate_statement(state, s)),
        ),
        syntax::IterationStatement::For(init, rest, s) => IterationStatement::For(
            translate_for_init(state, init),
            translate_for_rest(state, rest),
            Box::new(translate_statement(state, s)),
        ),
        syntax::IterationStatement::DoWhile(s, e) => IterationStatement::DoWhile(
            Box::new(translate_statement(state, s)),
            Box::new(translate_expression(state, e)),
        ),
    }
}

fn translate_case(state: &mut State, c: &syntax::CaseLabel) -> CaseLabel {
    match c {
        syntax::CaseLabel::Def => CaseLabel::Def,
        syntax::CaseLabel::Case(e) => CaseLabel::Case(Box::new(translate_expression(state, e))),
    }
}

fn translate_selection_rest(
    state: &mut State,
    s: &syntax::SelectionRestStatement,
) -> (Box<Statement>, Option<Box<Statement>>) {
    match s {
        syntax::SelectionRestStatement::Statement(s) => {
            (Box::new(translate_statement(state, s)), None)
        }
        syntax::SelectionRestStatement::Else(if_body, rest) => (
            Box::new(translate_statement(state, if_body)),
            Some(Box::new(translate_statement(state, rest))),
        ),
    }
}

fn translate_selection(state: &mut State, s: &syntax::SelectionStatement) -> SelectionStatement {
    let cond = Box::new(translate_expression(state, &s.cond));
    let (body, else_stmt) = translate_selection_rest(state, &s.rest);
    SelectionStatement {
        cond,
        body,
        else_stmt,
    }
}

fn translate_simple_statement(state: &mut State, s: &syntax::SimpleStatement) -> SimpleStatement {
    match s {
        syntax::SimpleStatement::Declaration(d) => {
            SimpleStatement::Declaration(translate_declaration(state, d, RunClass::Unknown))
        }
        syntax::SimpleStatement::Expression(e) => {
            SimpleStatement::Expression(e.as_ref().map(|e| translate_expression(state, e)))
        }
        syntax::SimpleStatement::Iteration(i) => {
            SimpleStatement::Iteration(translate_iteration(state, i))
        }
        syntax::SimpleStatement::Selection(s) => {
            SimpleStatement::Selection(translate_selection(state, s))
        }
        syntax::SimpleStatement::Jump(j) => SimpleStatement::Jump(translate_jump(state, j)),
        syntax::SimpleStatement::Switch(s) => SimpleStatement::Switch(translate_switch(state, s)),
        syntax::SimpleStatement::CaseLabel(_) => panic!("should be handled by translate_switch"),
    }
}

fn translate_statement(state: &mut State, s: &syntax::Statement) -> Statement {
    match s {
        syntax::Statement::Compound(s) => {
            Statement::Compound(Box::new(translate_compound_statement(state, s)))
        }
        syntax::Statement::Simple(s) => {
            Statement::Simple(Box::new(translate_simple_statement(state, s)))
        }
    }
}

fn translate_compound_statement(
    state: &mut State,
    cs: &syntax::CompoundStatement,
) -> CompoundStatement {
    CompoundStatement {
        statement_list: cs
            .statement_list
            .iter()
            .map(|x| translate_statement(state, x))
            .collect(),
    }
}

fn translate_function_parameter_declaration(
    state: &mut State,
    p: &syntax::FunctionParameterDeclaration,
    index: usize,
) -> FunctionParameterDeclaration {
    match p {
        syntax::FunctionParameterDeclaration::Named(qual, p) => {
            let mut ty: Type = lift(state, &p.ty);
            if let Some(a) = &p.ident.array_spec {
                ty.array_sizes = Some(Box::new(lift(state, a)));
            }

            ty.precision = get_precision(qual);

            let decl = SymDecl::Local(
                StorageClass::None,
                ty.clone(),
                RunClass::Dependent(1 << index),
            );
            let d = FunctionParameterDeclarator {
                ty,
                name: p.ident.ident.clone(),
                sym: state.declare(p.ident.ident.as_str(), decl),
            };
            FunctionParameterDeclaration::Named(lift_type_qualifier_for_parameter(state, qual), d)
        }
        syntax::FunctionParameterDeclaration::Unnamed(qual, p) => {
            FunctionParameterDeclaration::Unnamed(
                lift_type_qualifier_for_parameter(state, qual),
                p.clone(),
            )
        }
    }
}

fn translate_prototype(
    state: &mut State,
    cs: &syntax::FunctionPrototype,
) -> (FunctionPrototype, SymRef) {
    let prototype = FunctionPrototype {
        ty: lift(state, &cs.ty),
        name: cs.name.clone(),
        parameters: cs
            .parameters
            .iter()
            .enumerate()
            .map(|(i, x)| translate_function_parameter_declaration(state, x, i))
            .collect(),
    };
    let sym = if let Some(sym) = state.lookup(prototype.name.as_str()) {
        match &state.sym(sym).decl {
            SymDecl::UserFunction(..) => {}
            _ => panic!(
                "prototype conflicts with existing symbol: {}",
                prototype.name.as_str()
            ),
        }
        sym
    } else {
        let pfd = Rc::new(FunctionDefinition {
            prototype: prototype.clone(),
            body: CompoundStatement::new(),
            globals: Vec::new(),
            texel_fetches: HashMap::new(),
        });
        state.declare(
            prototype.name.as_str(),
            SymDecl::UserFunction(pfd, RunClass::Unknown),
        )
    };
    (prototype, sym)
}

fn translate_function_prototype(
    state: &mut State,
    prototype: &syntax::FunctionPrototype,
) -> FunctionPrototype {
    let (prototype, _) = translate_prototype(state, prototype);
    prototype
}

fn translate_function_definition(
    state: &mut State,
    sfd: &syntax::FunctionDefinition,
) -> Rc<FunctionDefinition> {
    let (prototype, sym) = translate_prototype(state, &sfd.prototype);

    state.push_scope(prototype.name.as_str().into());
    state.in_function = Some(sym);
    state.modified_globals.get_mut().clear();
    state.texel_fetches.clear();
    let body = translate_compound_statement(state, &sfd.statement);
    let mut globals = Vec::new();
    mem::swap(&mut globals, state.modified_globals.get_mut());
    let mut texel_fetches = HashMap::new();
    mem::swap(&mut texel_fetches, &mut state.texel_fetches);
    state.in_function = None;
    state.pop_scope();

    let fd = Rc::new(FunctionDefinition {
        prototype,
        body,
        globals,
        texel_fetches,
    });
    state.sym_mut(sym).decl = SymDecl::UserFunction(fd.clone(), RunClass::Unknown);
    fd
}

fn translate_external_declaration(
    state: &mut State,
    ed: &syntax::ExternalDeclaration,
) -> ExternalDeclaration {
    match ed {
        syntax::ExternalDeclaration::Declaration(d) => {
            ExternalDeclaration::Declaration(translate_declaration(state, d, RunClass::Unknown))
        }
        syntax::ExternalDeclaration::FunctionDefinition(fd) => {
            ExternalDeclaration::FunctionDefinition(translate_function_definition(state, fd))
        }
        syntax::ExternalDeclaration::Preprocessor(p) => {
            ExternalDeclaration::Preprocessor(p.clone())
        }
    }
}

fn declare_function(
    state: &mut State,
    name: &str,
    cxx_name: Option<&'static str>,
    ret: Type,
    params: Vec<Type>,
) {
    let sig = FunctionSignature { ret, params };
    match state.lookup_sym_mut(name) {
        Some(Symbol {
            decl: SymDecl::NativeFunction(f, ..),
            ..
        }) => f.signatures.push(sig),
        None => {
            state.declare(
                name,
                SymDecl::NativeFunction(
                    FunctionType {
                        signatures: NonEmpty::new(sig),
                    },
                    cxx_name,
                ),
            );
        }
        _ => panic!("overloaded function name {}", name),
    }
    //state.declare(name, Type::Function(FunctionType{ v}))
}

pub fn ast_to_hir(state: &mut State, tu: &syntax::TranslationUnit) -> TranslationUnit {
    // global scope
    state.push_scope("global".into());
    use TypeKind::*;
    declare_function(
        state,
        "vec2",
        Some("make_vec2"),
        Type::new(Vec2),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "vec2",
        Some("make_vec2"),
        Type::new(Vec2),
        vec![Type::new(IVec2)],
    );
    declare_function(
        state,
        "vec2",
        Some("make_vec2"),
        Type::new(Vec2),
        vec![Type::new(IVec3)],
    );
    declare_function(
        state,
        "vec3",
        Some("make_vec3"),
        Type::new(Vec3),
        vec![Type::new(Float), Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "vec3",
        Some("make_vec3"),
        Type::new(Vec3),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "vec3",
        Some("make_vec3"),
        Type::new(Vec3),
        vec![Type::new(Vec2), Type::new(Float)],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![Type::new(Vec3), Type::new(Float)],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
        ],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![Type::new(Vec2), Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![Type::new(Float), Type::new(Float), Type::new(Vec2)],
    );
    declare_function(
        state,
        "vec4",
        Some("make_vec4"),
        Type::new(Vec4),
        vec![Type::new(Vec4)],
    );

    declare_function(
        state,
        "bvec2",
        Some("make_bvec2"),
        Type::new(BVec2),
        vec![Type::new(Bool)],
    );
    declare_function(
        state,
        "bvec4",
        Some("make_bvec4"),
        Type::new(BVec4),
        vec![Type::new(BVec2), Type::new(BVec2)],
    );

    declare_function(
        state,
        "int",
        Some("make_int"),
        Type::new(Int),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "float",
        Some("make_float"),
        Type::new(Float),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "float",
        Some("make_float"),
        Type::new(Float),
        vec![Type::new(Int)],
    );
    declare_function(
        state,
        "int",
        Some("make_int"),
        Type::new(Int),
        vec![Type::new(UInt)],
    );
    declare_function(
        state,
        "uint",
        Some("make_uint"),
        Type::new(UInt),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "uint",
        Some("make_uint"),
        Type::new(UInt),
        vec![Type::new(Int)],
    );
    declare_function(
        state,
        "ivec2",
        Some("make_ivec2"),
        Type::new(IVec2),
        vec![Type::new(UInt), Type::new(UInt)],
    );
    declare_function(
        state,
        "ivec2",
        Some("make_ivec2"),
        Type::new(IVec2),
        vec![Type::new(Int), Type::new(Int)],
    );
    declare_function(
        state,
        "ivec2",
        Some("make_ivec2"),
        Type::new(IVec2),
        vec![Type::new(Vec2)],
    );
    declare_function(
        state,
        "ivec3",
        Some("make_ivec3"),
        Type::new(IVec3),
        vec![Type::new(IVec2), Type::new(Int)],
    );
    declare_function(
        state,
        "ivec4",
        Some("make_ivec4"),
        Type::new(IVec4),
        vec![
            Type::new(Int),
            Type::new(Int),
            Type::new(Int),
            Type::new(Int),
        ],
    );
    declare_function(
        state,
        "ivec4",
        Some("make_ivec4"),
        Type::new(IVec4),
        vec![Type::new(IVec2), Type::new(Int), Type::new(Int)],
    );

    declare_function(
        state,
        "mat2",
        Some("make_mat2"),
        Type::new(Mat2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "mat2",
        Some("make_mat2"),
        Type::new(Mat2),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "mat2",
        Some("make_mat2"),
        Type::new(Mat2),
        vec![Type::new(Mat4)],
    );
    declare_function(
        state,
        "mat3",
        Some("make_mat3"),
        Type::new(Mat3),
        vec![Type::new(Vec3), Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "mat3",
        Some("make_mat3"),
        Type::new(Mat3),
        vec![Type::new(Mat4)],
    );
    declare_function(
        state,
        "mat3",
        Some("make_mat3"),
        Type::new(Mat3),
        vec![
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
        ],
    );
    declare_function(
        state,
        "mat4",
        Some("make_mat4"),
        Type::new(Mat4),
        vec![
            Type::new(Vec4),
            Type::new(Vec4),
            Type::new(Vec4),
            Type::new(Vec4),
        ],
    );
    declare_function(
        state,
        "mat4",
        Some("make_mat4"),
        Type::new(Mat4),
        vec![
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
            Type::new(Float),
        ],
    );
    declare_function(state, "abs", None, Type::new(Vec2), vec![Type::new(Vec2)]);
    declare_function(state, "abs", None, Type::new(Vec3), vec![Type::new(Vec3)]);
    declare_function(state, "abs", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(
        state,
        "dot",
        None,
        Type::new(Float),
        vec![Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "dot",
        None,
        Type::new(Float),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "min",
        None,
        Type::new(Float),
        vec![Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "min",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "min",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3)],
    );

    declare_function(
        state,
        "max",
        None,
        Type::new(Float),
        vec![Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "max",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "max",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Float)],
    );
    declare_function(
        state,
        "max",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3)],
    );

    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2), Type::new(BVec2)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2), Type::new(Float)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec4),
        vec![Type::new(Vec4), Type::new(Vec4), Type::new(Vec4)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec4),
        vec![Type::new(Vec4), Type::new(Vec4), Type::new(Float)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3), Type::new(Float)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3), Type::new(BVec3)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Float),
        vec![Type::new(Float), Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "mix",
        None,
        Type::new(Vec4),
        vec![Type::new(Vec4), Type::new(Vec4), Type::new(BVec4)],
    );
    declare_function(
        state,
        "step",
        None,
        Type::new(Float),
        vec![Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "step",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "step",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "notEqual",
        None,
        Type::new(BVec4),
        vec![Type::new(IVec4), Type::new(IVec4)],
    );

    declare_function(
        state,
        "fwidth",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2)],
    );
    declare_function(state, "cos", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(state, "sin", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(state, "tan", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(state, "atan", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(state, "atan", None, Type::new(Float), vec![Type::new(Float), Type::new(Float)]);
    declare_function(
        state,
        "clamp",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Float), Type::new(Float)],
    );
    declare_function(
        state,
        "clamp",
        None,
        Type::new(Double),
        vec![Type::new(Double), Type::new(Double), Type::new(Double)],
    );
    declare_function(
        state,
        "clamp",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2), Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "clamp",
        None,
        Type::new(Vec3),
        vec![Type::new(Vec3), Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "clamp",
        None,
        Type::new(Vec4),
        vec![Type::new(Vec4), Type::new(Vec4), Type::new(Vec4)],
    );
    declare_function(
        state,
        "length",
        None,
        Type::new(Float),
        vec![Type::new(Vec2)],
    );
    declare_function(state, "pow", None, Type::new(Vec3), vec![Type::new(Vec3)]);
    declare_function(state, "pow", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(state, "exp", None, Type::new(Float), vec![Type::new(Float)]);
    declare_function(
        state,
        "inversesqrt",
        None,
        Type::new(Float),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "sqrt",
        None,
        Type::new(Float),
        vec![Type::new(Float)],
    );
    declare_function(
        state,
        "distance",
        None,
        Type::new(Float),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );

    declare_function(
        state,
        "lessThanEqual",
        None,
        Type::new(BVec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "lessThanEqual",
        None,
        Type::new(BVec3),
        vec![Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(
        state,
        "lessThanEqual",
        None,
        Type::new(BVec4),
        vec![Type::new(Vec4), Type::new(Vec4)],
    );
    declare_function(
        state,
        "lessThan",
        None,
        Type::new(BVec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "greaterThan",
        None,
        Type::new(BVec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "greaterThanEqual",
        None,
        Type::new(BVec2),
        vec![Type::new(Vec2), Type::new(Vec2)],
    );
    declare_function(
        state,
        "greaterThanEqual",
        None,
        Type::new(BVec2),
        vec![Type::new(Vec4), Type::new(Vec4)],
    );
    declare_function(state, "any", None, Type::new(Bool), vec![Type::new(BVec2)]);
    declare_function(state, "all", None, Type::new(Bool), vec![Type::new(BVec2)]);
    declare_function(state, "all", None, Type::new(Bool), vec![Type::new(BVec4)]);

    declare_function(
        state,
        "if_then_else",
        None,
        Type::new(Vec3),
        vec![Type::new(BVec3), Type::new(Vec3), Type::new(Vec3)],
    );
    declare_function(state, "floor", None, Type::new(Vec4), vec![Type::new(Vec4)]);
    declare_function(state, "floor", None, Type::new(Vec2), vec![Type::new(Vec2)]);
    declare_function(
        state,
        "floor",
        None,
        Type::new(Double),
        vec![Type::new(Double)],
    );
    declare_function(
        state,
        "ceil",
        None,
        Type::new(Double),
        vec![Type::new(Double)],
    );
    declare_function(
        state,
        "round",
        None,
        Type::new(Double),
        vec![Type::new(Double)],
    );
    declare_function(
        state,
        "fract",
        None,
        Type::new(Float),
        vec![Type::new(Float)],
    );
    declare_function(state, "mod", None, Type::new(Vec2), vec![Type::new(Vec2)]);
    declare_function(state, "mod", None, Type::new(Float), vec![Type::new(Float)]);

    declare_function(
        state,
        "texelFetch",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2D), Type::new(IVec2), Type::new(Int)],
    );
    declare_function(
        state,
        "texelFetch",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2DArray), Type::new(IVec3), Type::new(Int)],
    );
    declare_function(
        state,
        "texelFetch",
        None,
        Type::new(IVec4),
        vec![Type::new(ISampler2D), Type::new(IVec2), Type::new(Int)],
    );
    declare_function(
        state,
        "texelFetchOffset",
        None,
        Type::new(Vec4),
        vec![
            Type::new(Sampler2D),
            Type::new(IVec2),
            Type::new(Int),
            Type::new(IVec2),
        ],
    );
    declare_function(
        state,
        "texelFetchOffset",
        None,
        Type::new(IVec4),
        vec![
            Type::new(ISampler2D),
            Type::new(IVec2),
            Type::new(Int),
            Type::new(IVec2),
        ],
    );
    declare_function(
        state,
        "texture",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2D), Type::new(Vec2)],
    );
    declare_function(
        state,
        "texture",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2DRect), Type::new(Vec2)],
    );
    declare_function(
        state,
        "texture",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2DArray), Type::new(Vec3)],
    );
    declare_function(
        state,
        "textureLod",
        None,
        Type::new(Vec4),
        vec![Type::new(Sampler2DArray), Type::new(Vec3), Type::new(Float)],
    );
    declare_function(
        state,
        "textureSize",
        None,
        Type::new(IVec3),
        vec![Type::new(Sampler2DArray), Type::new(Int)],
    );
    declare_function(
        state,
        "textureSize",
        None,
        Type::new(IVec2),
        vec![Type::new(Sampler2D), Type::new(Int)],
    );
    declare_function(
        state,
        "textureSize",
        None,
        Type::new(IVec2),
        vec![Type::new(Sampler2DRect), Type::new(Int)],
    );

    declare_function(
        state,
        "inverse",
        None,
        Type::new(Mat2),
        vec![Type::new(Mat2)],
    );
    declare_function(
        state,
        "transpose",
        None,
        Type::new(Mat3),
        vec![Type::new(Mat3)],
    );
    declare_function(
        state,
        "normalize",
        None,
        Type::new(Vec2),
        vec![Type::new(Vec2)],
    );
    state.declare(
        "gl_FragCoord",
        SymDecl::Global(StorageClass::In, None, Type::new(Vec4), RunClass::Vector),
    );
    state.declare(
        "gl_FragColor",
        SymDecl::Global(StorageClass::Out, None, Type::new(Vec4), RunClass::Vector),
    );
    state.declare(
        "gl_Position",
        SymDecl::Global(StorageClass::Out, None, Type::new(Vec4), RunClass::Vector),
    );

    TranslationUnit(tu.0.map(state, translate_external_declaration))
}

fn infer_expr_inner(state: &mut State, expr: &Expr, assign: &mut SymRef) -> RunClass {
    match expr.kind {
        ExprKind::Variable(ref i) => {
            *assign = *i;
            match &state.sym(*i).decl {
                SymDecl::Local(_, _, ref run_class) => *run_class,
                SymDecl::Global(_, _, _, ref run_class) => *run_class,
                _ => panic!(),
            }
        }
        ExprKind::IntConst(_)
        | ExprKind::UIntConst(_)
        | ExprKind::BoolConst(_)
        | ExprKind::FloatConst(_)
        | ExprKind::DoubleConst(_) => RunClass::Scalar,
        ExprKind::Unary(_, ref e) => infer_expr(state, e),
        ExprKind::Binary(_, ref l, ref r) => infer_expr(state, l).merge(infer_expr(state, r)),
        ExprKind::Ternary(ref c, ref s, ref e) => infer_expr(state, c)
            .merge(infer_expr(state, s))
            .merge(infer_expr(state, e)),
        ExprKind::Assignment(ref v, _, ref e) => {
            let mut sym = SymRef(!0);
            let run_class = infer_expr_inner(state, v, &mut sym).merge(infer_expr(state, e));
            assert!(sym != SymRef(!0));
            state.merge_run_class(sym, run_class)
        }
        ExprKind::Bracket(ref e, ref indx) => {
            infer_expr_inner(state, e, assign).merge(infer_expr(state, indx))
        }
        ExprKind::FunCall(ref fun, ref args) => {
            let arg_classes: Vec<(RunClass, SymRef)> = args
                .iter()
                .map(|e| {
                    let mut assign = SymRef(!0);
                    let run_class = infer_expr_inner(state, e, &mut assign);
                    (run_class, assign)
                })
                .collect();
            let run_class = if args.is_empty() {
                RunClass::Scalar
            } else {
                arg_classes
                    .iter()
                    .fold(RunClass::Unknown, |x, &(y, _)| x.merge(y))
            };
            match fun {
                FunIdentifier::Identifier(ref sym) => match &state.sym(*sym).decl {
                    SymDecl::NativeFunction(..) => run_class,
                    SymDecl::UserFunction(ref fd, ref run_class) => {
                        for (&(mut arg_class, assign), param) in
                            arg_classes.iter().zip(fd.prototype.parameters.iter())
                        {
                            if let FunctionParameterDeclaration::Named(Some(qual), p) = param {
                                match qual {
                                    ParameterQualifier::InOut | ParameterQualifier::Out => {
                                        if let SymDecl::Local(_, _, param_class) =
                                            &state.sym(p.sym).decl
                                        {
                                            match param_class {
                                                RunClass::Unknown | RunClass::Vector => {
                                                    arg_class = RunClass::Vector;
                                                }
                                                RunClass::Dependent(mask) => {
                                                    for i in 0 .. 31 {
                                                        if (mask & (1 << i)) != 0 {
                                                            arg_class =
                                                                arg_class.merge(arg_classes[i].0);
                                                        }
                                                    }
                                                }
                                                RunClass::Scalar => {}
                                            }
                                        }
                                        assert!(assign != SymRef(!0));
                                        state.merge_run_class(assign, arg_class);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if fd.prototype.ty.kind == TypeKind::Void {
                            RunClass::Scalar
                        } else {
                            match *run_class {
                                RunClass::Unknown | RunClass::Vector => RunClass::Vector,
                                RunClass::Dependent(mask) => {
                                    let mut ret_class = RunClass::Unknown;
                                    for i in 0 .. 31 {
                                        if (mask & (1 << i)) != 0 {
                                            ret_class = ret_class.merge(arg_classes[i].0);
                                        }
                                    }
                                    ret_class
                                }
                                RunClass::Scalar => RunClass::Scalar,
                            }
                        }
                    }
                    SymDecl::Struct(..) => run_class,
                    _ => panic!(),
                },
                FunIdentifier::Constructor(..) => run_class,
            }
        }
        ExprKind::Dot(ref e, _) => infer_expr_inner(state, e, assign),
        ExprKind::SwizzleSelector(ref e, _) => infer_expr_inner(state, e, assign),
        ExprKind::PostInc(ref e) => infer_expr_inner(state, e, assign),
        ExprKind::PostDec(ref e) => infer_expr_inner(state, e, assign),
        ExprKind::Comma(ref a, ref b) => {
            infer_expr(state, a);
            infer_expr(state, b)
        }
        ExprKind::Cond(_, ref e) => infer_expr(state, e),
        ExprKind::CondMask => RunClass::Vector,
    }
}

fn infer_expr(state: &mut State, expr: &Expr) -> RunClass {
    infer_expr_inner(state, expr, &mut SymRef(!0))
}

fn infer_condition(state: &mut State, c: &Condition) {
    match *c {
        Condition::Expr(ref e) => {
            infer_expr(state, e);
        }
    }
}

fn infer_iteration_statement(state: &mut State, ist: &IterationStatement) {
    let changed = state.run_class_changed.replace(true);
    match *ist {
        IterationStatement::While(ref cond, ref body) => {
            while state.run_class_changed.replace(false) {
                infer_condition(state, cond);
                infer_statement(state, body);
            }
        }
        IterationStatement::DoWhile(ref body, ref cond) => {
            while state.run_class_changed.replace(false) {
                infer_statement(state, body);
                infer_expr(state, cond);
            }
        }
        IterationStatement::For(ref init, ref rest, ref body) => {
            match *init {
                ForInitStatement::Expression(ref expr) => {
                    if let Some(ref e) = *expr {
                        infer_expr(state, e);
                    }
                }
                ForInitStatement::Declaration(ref d) => {
                    infer_declaration(state, d);
                }
            }
            while state.run_class_changed.replace(false) {
                if let Some(ref cond) = rest.condition {
                    infer_condition(state, cond);
                }
                if let Some(ref e) = rest.post_expr {
                    infer_expr(state, e);
                }
                infer_statement(state, body);
            }
        }
    }
    state.run_class_changed.set(changed);
}

fn infer_selection_statement(state: &mut State, sst: &SelectionStatement) {
    let mut branch_run_class = state.branch_run_class.merge(infer_expr(state, &sst.cond));
    mem::swap(&mut state.branch_run_class, &mut branch_run_class);
    let branch_declaration = state.branch_declaration;
    state.branch_declaration = state.last_declaration;
    infer_statement(state, &sst.body);
    if let Some(ref else_st) = sst.else_stmt {
        infer_statement(state, else_st);
    }
    state.branch_run_class = branch_run_class;
    state.branch_declaration = branch_declaration;
}

fn infer_expression_statement(state: &mut State, est: &ExprStatement) {
    if let Some(ref e) = *est {
        infer_expr(state, e);
    }
}

fn infer_switch_statement(state: &mut State, sst: &SwitchStatement) {
    let mut branch_run_class = state.branch_run_class.merge(infer_expr(state, &sst.head));
    mem::swap(&mut state.branch_run_class, &mut branch_run_class);
    let branch_declaration = state.branch_declaration;
    state.branch_declaration = state.last_declaration;
    for case in &sst.cases {
        for st in &case.stmts {
            infer_statement(state, st);
        }
    }
    state.branch_run_class = branch_run_class;
    state.branch_declaration = branch_declaration;
}

fn infer_jump_statement(state: &mut State, j: &JumpStatement) {
    match *j {
        JumpStatement::Continue => {}
        JumpStatement::Break => {}
        JumpStatement::Discard => {}
        JumpStatement::Return(ref e) => {
            if let Some(e) = e {
                let run_class = infer_expr(state, e);
                state.return_run_class(run_class);
            }
        }
    }
}

fn infer_initializer(state: &mut State, i: &Initializer) -> RunClass {
    match *i {
        Initializer::Simple(ref e) => infer_expr(state, e),
        Initializer::List(ref list) => {
            let mut run_class = RunClass::Unknown;
            for ini in list.0.iter() {
                run_class = run_class.merge(infer_initializer(state, ini));
            }
            run_class
        }
    }
}

fn infer_declaration(state: &mut State, d: &Declaration) {
    match *d {
        Declaration::FunctionPrototype(..) => {}
        Declaration::InitDeclaratorList(ref list) => {
            state.last_declaration = list.head.name;

            let mut run_class = RunClass::Unknown;
            for decl in &list.tail {
                if let Some(ref initializer) = decl.initializer {
                    run_class = run_class.merge(infer_initializer(state, initializer));
                }
            }
            if let Some(ref initializer) = list.head.initializer {
                run_class = run_class.merge(infer_initializer(state, initializer));
                state.merge_run_class(list.head.name, run_class);
            }
        }
        Declaration::Precision(..) => {}
        Declaration::Block(..) => {}
        Declaration::Global(..) => {}
        Declaration::StructDefinition(..) => {}
    }
}

fn infer_simple_statement(state: &mut State, sst: &SimpleStatement) {
    match *sst {
        SimpleStatement::Declaration(ref d) => infer_declaration(state, d),
        SimpleStatement::Expression(ref e) => infer_expression_statement(state, e),
        SimpleStatement::Selection(ref s) => infer_selection_statement(state, s),
        SimpleStatement::Switch(ref s) => infer_switch_statement(state, s),
        SimpleStatement::Iteration(ref i) => infer_iteration_statement(state, i),
        SimpleStatement::Jump(ref j) => infer_jump_statement(state, j),
    }
}

fn infer_compound_statement(state: &mut State, cst: &CompoundStatement) {
    for st in &cst.statement_list {
        infer_statement(state, st);
    }
}

fn infer_statement(state: &mut State, st: &Statement) {
    match *st {
        Statement::Compound(ref cst) => infer_compound_statement(state, cst),
        Statement::Simple(ref sst) => infer_simple_statement(state, sst),
    }
}

fn infer_function_definition(state: &mut State, fd: &FunctionDefinition) {
    state.in_function = Some(state.lookup(fd.prototype.name.as_str()).unwrap());

    state.run_class_changed.set(true);
    while state.run_class_changed.replace(false) {
        for st in &fd.body.statement_list {
            infer_statement(state, st);
        }
    }

    state.in_function = None;
}

fn infer_external_declaration(state: &mut State, ed: &ExternalDeclaration) {
    match *ed {
        ExternalDeclaration::Preprocessor(_) => {}
        ExternalDeclaration::FunctionDefinition(ref fd) => infer_function_definition(state, fd),
        ExternalDeclaration::Declaration(_) => {}
    }
}

pub fn infer_run_class(state: &mut State, tu: &TranslationUnit) {
    for ed in &(tu.0).0 {
        infer_external_declaration(state, ed);
    }
}
