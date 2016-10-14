/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use owning_ref::RcRef;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use style::owning_handle::OwningHandle;

#[test]
fn owning_handle() {
    use std::cell::RefCell;
    let cell = Rc::new(RefCell::new(2));
    let cell_ref = RcRef::new(cell);
    let mut handle = OwningHandle::new(cell_ref, |x| unsafe { x.as_ref() }.unwrap().borrow_mut());
    assert_eq!(*handle, 2);
    *handle = 3;
    assert_eq!(*handle, 3);
}

#[test]
fn nested() {
    let result = {
        let complex = Rc::new(RefCell::new(Arc::new(RwLock::new("someString"))));
        let curr = RcRef::new(complex);
        let curr = OwningHandle::new(curr, |x| unsafe { x.as_ref() }.unwrap().borrow_mut());
        let mut curr = OwningHandle::new(curr, |x| unsafe { x.as_ref() }.unwrap().try_write().unwrap());
        assert_eq!(*curr, "someString");
        *curr = "someOtherString";
        curr
    };
    assert_eq!(*result, "someOtherString");
}
