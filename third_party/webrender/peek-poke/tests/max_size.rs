// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

use peek_poke::{PeekPoke, Poke};
use std::{marker::PhantomData, mem::size_of};

#[test]
fn test_numbers() {
    assert_eq!(u8::max_size(), size_of::<u8>());
    assert_eq!(u16::max_size(), size_of::<u16>());
    assert_eq!(u32::max_size(), size_of::<u32>());
    assert_eq!(u64::max_size(), size_of::<u64>());
    assert_eq!(usize::max_size(), size_of::<usize>());
    assert_eq!(i8::max_size(), size_of::<i8>());
    assert_eq!(i16::max_size(), size_of::<i16>());
    assert_eq!(i32::max_size(), size_of::<i32>());
    assert_eq!(i64::max_size(), size_of::<i64>());
    assert_eq!(isize::max_size(), size_of::<isize>());
    // floating
    assert_eq!(f32::max_size(), size_of::<f32>());
    assert_eq!(f64::max_size(), size_of::<f64>());
}

#[test]
fn test_bool() {
    assert_eq!(bool::max_size(), size_of::<u8>());
}

#[test]
fn test_option() {
    assert_eq!(
        Option::<usize>::max_size(),
        <u8>::max_size() + <usize>::max_size()
    );
}

#[test]
fn test_fixed_size_array() {
    assert_eq!(<[u32; 32]>::max_size(), 32 * size_of::<u32>());
    assert_eq!(<[u64; 8]>::max_size(), 8 * size_of::<u64>());
    assert_eq!(<[u8; 19]>::max_size(), 19 * size_of::<u8>());
}

#[test]
fn test_tuple() {
    assert_eq!(<(isize, )>::max_size(), size_of::<isize>());
    assert_eq!(<(isize, isize, isize)>::max_size(), 3 * size_of::<isize>());
    assert_eq!(<(isize, ())>::max_size(), size_of::<isize>());
}

#[test]
fn test_basic_struct() {
    #[derive(Debug, PeekPoke)]
    struct Bar {
        a: u32,
        b: u32,
        c: u32,
    }

    assert_eq!(<Bar>::max_size(), 3 * <u32>::max_size());
}

#[test]
fn test_enum() {
    #[derive(Clone, Copy, PeekPoke)]
    enum TestEnum {
        NoArg,
        OneArg(usize),
        Args(usize, usize),
        AnotherNoArg,
        StructLike { x: usize, y: f32 },
    }
    assert_eq!(
        TestEnum::max_size(),
        <u8>::max_size() + 2 * <usize>::max_size()
    );
}

#[test]
fn test_enum_cstyle() {
    #[repr(u32)]
    #[derive(Clone, Copy, PeekPoke)]
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
    assert_eq!(BorderStyle::max_size(), <u8>::max_size());
}

#[test]
fn test_phantom_data() {
    struct Bar;
    #[derive(PeekPoke)]
    struct Foo {
        x: u32,
        y: u32,
        _marker: PhantomData<Bar>,
    }
    assert_eq!(Foo::max_size(), 2 * size_of::<u32>())
}
