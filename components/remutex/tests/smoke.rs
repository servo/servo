// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// These tests came from https://github.com/rust-lang/rust/blob/master/src/libstd/sys/common/remutex.rs

extern crate servo_remutex;

use servo_remutex::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;

#[test]
fn smoke() {
    let m = ReentrantMutex::new(());
    {
        let a = m.lock().unwrap();
        {
            let b = m.lock().unwrap();
            {
                let c = m.lock().unwrap();
                assert_eq!(*c, ());
            }
            assert_eq!(*b, ());
        }
        assert_eq!(*a, ());
    }
}

#[test]
fn is_mutex() {
    let m = Arc::new(ReentrantMutex::new(RefCell::new(0)));
    let m2 = m.clone();
    let lock = m.lock().unwrap();
    let child = thread::spawn(move || {
        let lock = m2.lock().unwrap();
        assert_eq!(*lock.borrow(), 4950);
    });
    for i in 0..100 {
        let lock = m.lock().unwrap();
        *lock.borrow_mut() += i;
    }
    drop(lock);
    child.join().unwrap();
}

#[test]
fn trylock_works() {
    let m = Arc::new(ReentrantMutex::new(()));
    let m2 = m.clone();
    let _lock = m.try_lock().unwrap();
    let _lock2 = m.try_lock().unwrap();
    thread::spawn(move || {
        let lock = m2.try_lock();
        assert!(lock.is_err());
    }).join().unwrap();
    let _lock3 = m.try_lock().unwrap();
}

pub struct Answer<'a>(pub ReentrantMutexGuard<'a, RefCell<u32>>);
impl<'a> Drop for Answer<'a> {
    fn drop(&mut self) {
        *self.0.borrow_mut() = 42;
    }
}

#[test]
fn poison_works() {
    let m = Arc::new(ReentrantMutex::new(RefCell::new(0)));
    let mc = m.clone();
    let result = thread::spawn(move ||{
        let lock = mc.lock().unwrap();
        *lock.borrow_mut() = 1;
        let lock2 = mc.lock().unwrap();
        *lock.borrow_mut() = 2;
        let _answer = Answer(lock2);
        println!("Intentionally panicking.");
        panic!("What the answer to my lifetimes dilemma is?");
    }).join();
    assert!(result.is_err());
    let r = m.lock().err().unwrap().into_inner();
    assert_eq!(*r.borrow(), 42);
}

