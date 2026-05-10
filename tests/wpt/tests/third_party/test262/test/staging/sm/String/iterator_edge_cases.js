// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/

// Test that we can't confuse %StringIteratorPrototype% for a
// StringIterator object.
function TestStringIteratorPrototypeConfusion() {
    var iter = ""[Symbol.iterator]();
    assert.throws(
        TypeError,
        () => iter.next.call(Object.getPrototypeOf(iter)),
        "next method called on incompatible String Iterator");
}
TestStringIteratorPrototypeConfusion();

// Tests that we can use %StringIteratorPrototype%.next on a
// cross-compartment iterator.
function TestStringIteratorWrappers() {
    var iter = ""[Symbol.iterator]();
    assert.deepEqual(iter.next.call($262.createRealm().global.eval('"x"[Symbol.iterator]()')),
		 { value: "x", done: false })
}
TestStringIteratorWrappers();
