// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use peek_poke::{Peek, PeekPoke, Poke};
use std::{fmt::Debug, marker::PhantomData};

fn poke_into<V: Peek + Poke>(a: &V) -> Vec<u8> {
    let mut v = <Vec<u8>>::with_capacity(<V>::max_size());
    let end_ptr = unsafe { a.poke_into(v.as_mut_ptr()) };
    let new_size = end_ptr as usize - v.as_ptr() as usize;
    assert!(new_size <= v.capacity());
    unsafe {
        v.set_len(new_size);
    }
    v
}

#[cfg(not(feature = "option_copy"))]
fn the_same<V>(a: V)
where
    V: Debug + Default + PartialEq + Peek + Poke,
{
    let v = poke_into(&a);
    let (b, end_ptr) = unsafe { peek_poke::peek_from_default(v.as_ptr()) };
    let size = end_ptr as usize - v.as_ptr() as usize;
    assert_eq!(size, v.len());
    assert_eq!(a, b);
}

#[cfg(feature = "option_copy")]
fn the_same<V>(a: V)
where
    V: Copy + Debug + PartialEq + Peek + Poke,
{
    let v = poke_into(&a);
    let mut b = a;
    let end_ptr = unsafe { b.peek_from(v.as_ptr()) };
    let size = end_ptr as usize - v.as_ptr() as usize;
    assert_eq!(size, v.len());
    assert_eq!(a, b);
}

#[test]
fn test_numbers() {
    // unsigned positive
    the_same(5u8);
    the_same(5u16);
    the_same(5u32);
    the_same(5u64);
    the_same(5usize);
    // signed positive
    the_same(5i8);
    the_same(5i16);
    the_same(5i32);
    the_same(5i64);
    the_same(5isize);
    // signed negative
    the_same(-5i8);
    the_same(-5i16);
    the_same(-5i32);
    the_same(-5i64);
    the_same(-5isize);
    // floating
    the_same(-100f32);
    the_same(0f32);
    the_same(5f32);
    the_same(-100f64);
    the_same(5f64);
}

#[test]
fn test_bool() {
    the_same(true);
    the_same(false);
}

#[cfg(any(feature = "option_copy", feature = "option_default"))]
#[test]
fn test_option() {
    the_same(Some(5usize));
    //the_same(Some("foo bar".to_string()));
    the_same(None::<usize>);
}

#[test]
fn test_fixed_size_array() {
    the_same([24u32; 32]);
    the_same([1u64, 2, 3, 4, 5, 6, 7, 8]);
    the_same([0u8; 19]);
}

#[test]
fn test_tuple() {
    the_same((1isize, ));
    the_same((1isize, 2isize, 3isize));
    the_same((1isize, ()));
}

#[test]
fn test_basic_struct() {
    #[derive(Copy, Clone, Debug, Default, PartialEq, PeekPoke)]
    struct Bar {
        a: u32,
        b: u32,
        c: u32,
        #[cfg(any(feature = "option_copy", feature = "option_default"))]
        d: Option<u32>,
    }

    the_same(Bar {
        a: 2,
        b: 4,
        c: 42,
        #[cfg(any(feature = "option_copy", feature = "option_default"))]
        d: None,
    });
}

#[test]
fn test_enum() {
    #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
    enum TestEnum {
        NoArg,
        OneArg(usize),
        Args(usize, usize),
        AnotherNoArg,
        StructLike { x: usize, y: f32 },
    }

    impl Default for TestEnum {
        fn default() -> Self {
            TestEnum::NoArg
        }
    }

    the_same(TestEnum::NoArg);
    the_same(TestEnum::OneArg(4));
    the_same(TestEnum::Args(4, 5));
    the_same(TestEnum::AnotherNoArg);
    the_same(TestEnum::StructLike { x: 4, y: 3.14159 });
}

#[test]
fn test_enum_cstyle() {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PeekPoke)]
    enum BorderStyle {
        None = 0,
        Solid = 1,
        Double = 2,
        Dotted = 3,
        Dashed = 4,
        Hidden = 5,
        Groove = 6,
        Ridge = 7,
        Inset = 8,
        Outset = 9,
    }

    impl Default for BorderStyle {
        fn default() -> Self {
            BorderStyle::None
        }
    }

    the_same(BorderStyle::None);
    the_same(BorderStyle::Solid);
    the_same(BorderStyle::Double);
    the_same(BorderStyle::Dotted);
    the_same(BorderStyle::Dashed);
    the_same(BorderStyle::Hidden);
    the_same(BorderStyle::Groove);
    the_same(BorderStyle::Ridge);
    the_same(BorderStyle::Inset);
    the_same(BorderStyle::Outset);
}

#[test]
fn test_phantom_data() {
    struct Bar;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PeekPoke)]
    struct Foo {
        x: u32,
        y: u32,
        _marker: PhantomData<Bar>,
    }
    the_same(Foo {
        x: 19,
        y: 42,
        _marker: PhantomData,
    });
}

#[test]
fn test_generic() {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PeekPoke)]
    struct Foo<T> {
        x: T,
        y: T,
    }
    the_same(Foo { x: 19.0, y: 42.0 });
}

#[test]
fn test_generic_enum() {
    #[derive(Clone, Copy, Debug, Default, PartialEq, PeekPoke)]
    pub struct PropertyBindingKey<T> {
        pub id: usize,
        _phantom: PhantomData<T>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
    pub enum PropertyBinding<T> {
        Value(T),
        Binding(PropertyBindingKey<T>, T),
    }

    impl<T: Default> Default for PropertyBinding<T> {
        fn default() -> Self {
            PropertyBinding::Value(Default::default())
        }
    }
}

#[cfg(all(feature = "extras", feature = "option_copy"))]
mod extra_tests {
    use super::*;
    use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Transform3D, Vector2D};
    use std::mem::size_of;

    #[test]
    fn euclid_types() {
        the_same(Point2D::<f32>::new(1.0, 2.0));
        assert_eq!(Point2D::<f32>::max_size(), 2 * size_of::<f32>());

        the_same(Rect::<f32>::new(
            Point2D::<f32>::new(0.0, 0.0),
            Size2D::<f32>::new(100.0, 80.0),
        ));
        assert_eq!(Rect::<f32>::max_size(), 4 * size_of::<f32>());

        the_same(SideOffsets2D::<f32>::new(0.0, 10.0, -1.0, -10.0));
        assert_eq!(SideOffsets2D::<f32>::max_size(), 4 * size_of::<f32>());

        the_same(Transform3D::<f32>::identity());
        assert_eq!(Transform3D::<f32>::max_size(), 16 * size_of::<f32>());

        the_same(Vector2D::<f32>::new(1.0, 2.0));
        assert_eq!(Vector2D::<f32>::max_size(), 2 * size_of::<f32>());
    }

    #[test]
    fn webrender_api_types() {
        type PipelineSourceId = i32;
        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct PipelineId(pub PipelineSourceId, pub u32);

        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct ClipChainId(pub u64, pub PipelineId);

        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct SpatialId(pub usize, pub PipelineId);

        the_same(PipelineId(42, 2));
        the_same(ClipChainId(19u64, PipelineId(42, 2)));
        the_same(SpatialId(19usize, PipelineId(42, 2)));
    }
}
