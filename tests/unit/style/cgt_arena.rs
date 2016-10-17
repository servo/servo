/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::thread;
use style::atomic_refcell::AtomicRefCell;
use style::cgt_arena::{Arena, ArenaArc, ITEMS_PER_LIST, Token, TokenData};
use test::Bencher;

#[test]
fn semantics() {
    let arena = Arena::<usize>::new();
    let mut td1;
    let td2;
    {
        let token = arena.borrow().allocate();
        *token.borrow_mut() = 42;
        assert_eq!(*token.borrow(), 42);
        td1 = token.data();
    }
    {
        let token = unsafe { Token::create(td1, arena.clone()) }.unwrap();
        assert_eq!(*token.borrow(), 42);
    }
    arena.borrow_mut().reset();
    {
        let token = unsafe { Token::create(td1, arena.clone()) };
        assert!(token.is_none());
        td1 = arena.borrow().allocate_raw();
        let token2 = arena.borrow().allocate();
        *token2.borrow_mut() = 30;
        td2 = token2.data();
    }
    {
        let token1 = unsafe { Token::create(td1, arena.clone()) }.unwrap();
        let token2 = unsafe { Token::create(td2, arena.clone()) }.unwrap();
        let token2_dup = unsafe { Token::create(td2, arena.clone()) }.unwrap();
        assert_eq!(*token1.borrow(), 0);
        assert_eq!(*token2.borrow(), 30);
        assert_eq!(*token2_dup.borrow(), 30);
        *token1.borrow_mut() = 31;
        assert_eq!(*token1.borrow(), 31);
        *token2.borrow_mut() = 50;
        assert_eq!(*token2.borrow(), 50);
        assert_eq!(*token2_dup.borrow(), 50);

    }
    arena.borrow_mut().reset();
    assert!(unsafe { Token::create(td1, arena.clone()) }.is_none());
    assert!(unsafe { Token::create(td2, arena.clone()) }.is_none());
}

struct Foo {
    pub a: i64,
    pub b: Vec<u32>,
    pub c: Box<u64>,
    pub index: u32,
    pub generation: u32,
}

impl Default for Foo {
    fn default() -> Foo {
        Foo {
            a: -42,
            b: vec![5, 10, 15],
            c: Box::new(42),
            index: 0,
            generation: 0,
        }
    }
}

const STRESS_ALLOCATIONS: u32 = (ITEMS_PER_LIST * 4 + 4) as u32;
const NUM_THREADS: usize = 4;

fn make_allocations(arena: &ArenaArc<Foo>, generation: u32)
                    -> Vec<(u32, TokenData)> {
    let mut allocations = Vec::new();
    for x in 0..STRESS_ALLOCATIONS {
        let token = arena.borrow().allocate();
        let mut f = token.borrow_mut();
        assert_eq!((*f).a, -42);
        assert_eq!((*f).b[1], 10);
        assert_eq!(*(*f).c, 42);
        (*f).index = x as u32;
        (*f).generation = generation;
        allocations.push((x, token.data()));
    }
    allocations
}

fn verify_allocations(arena: &ArenaArc<Foo>,
                      generation: u32,
                      allocations: &[(u32, TokenData)])
{
    for x in allocations.iter() {
        let token = unsafe {
            Token::create(x.1, arena.clone())
        }.unwrap();
        let f = token.borrow();
        assert_eq!((*f).index, x.0);
        assert_eq!((*f).generation, generation);
    }
}

fn verify_expired(arena: &ArenaArc<Foo>,
                  allocations: &[(u32, TokenData)]) {
    for x in allocations.iter() {
        assert!(x.1.is_expired(&arena.borrow()));
        let maybe_token = unsafe {
            Token::create(x.1.clone(), arena.clone())
        };
        assert!(maybe_token.is_none());
    }
}

#[test]
fn stress_sequential() {
    let arena = Arena::<Foo>::new();
    for generation in 1_u32..4_u32 {
        let allocations = make_allocations(&arena, generation);
        verify_allocations(&arena, generation, &allocations);
        arena.borrow_mut().reset();
        verify_expired(&arena, &allocations);
    }
}

#[test]
fn stress_parallel() {
    let arena = Arena::<Foo>::new();
    for generation in 1_u32..4_u32 {
        let mut threads = Vec::new();
        let mut allocations: Vec<(u32, TokenData)> = Vec::new();

        // Allocate concurrently.
        for _ in 0..NUM_THREADS {
            let a = arena.clone();
            let fun = move || make_allocations(&a, generation);
            threads.push(thread::spawn(fun));
        }
        for thread in threads.into_iter() {
            allocations.extend(thread.join().unwrap());
        }

        // Verify all allocations concurrently on multiple threads.
        let mut threads = Vec::new();
        for _ in 0..NUM_THREADS {
            let a = arena.clone();
            let allocs = allocations.clone();
            let fun = move || verify_allocations(&a, generation, &allocs);
            threads.push(thread::spawn(fun));
        }
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }

        // Reset arena.
        arena.borrow_mut().reset();

        // Verify expiration.
        let mut threads = Vec::new();
        for _ in 0..NUM_THREADS {
            let a = arena.clone();
            let allocs = allocations.clone();
            let fun = move || verify_expired(&a, &allocs);
            threads.push(thread::spawn(fun));
        }
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }
    }
}

#[test]
fn zero_is_valid_td() {
    let arena = Arena::<usize>::new();
    let data = TokenData::deserialize(0_u64);
    let token = unsafe { Token::create(data, arena.clone()) };
    assert!(token.is_none());
}

struct Bar {
    pub maybe_foo: Option<Foo>,
    pub a: i64,
    pub b: u32,
    pub c: u64,
    pub d: u32,
}

impl Default for Bar {
    fn default() -> Bar {
        Bar {
            maybe_foo: None,
            a: -42,
            b: 10,
            c: 42,
            d: 0,
        }
    }
}

#[bench]
fn sequential_stack_allocations(b: &mut Bencher) {
    b.iter(|| AtomicRefCell::new(Bar::default()));
}

#[bench]
fn sequential_allocations_box(b: &mut Bencher) {
    b.iter(|| Box::new(AtomicRefCell::new(Bar::default())));
}

#[bench]
fn sequential_allocations_arena(b: &mut Bencher) {
    let arena = Arena::<Bar>::new();
    b.iter(|| arena.borrow().allocate());
}

#[bench]
fn sequential_allocations_arena_preborrowed(b: &mut Bencher) {
    let arena = Arena::<Bar>::new();
    let borrowed = arena.borrow();
    b.iter(|| borrowed.allocate());
}

#[bench]
fn sequential_raw_allocations_arena_preborrowed(b: &mut Bencher) {
    let arena = Arena::<Bar>::new();
    let borrowed = arena.borrow();
    b.iter(|| borrowed.allocate_raw());
}
