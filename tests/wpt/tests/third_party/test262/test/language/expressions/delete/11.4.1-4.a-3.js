// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test is actually testing the [[Delete]] internal method (8.12.8). Since the
    language provides no way to directly exercise [[Delete]], the tests are placed here.
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns false when deleting a non-configurable
    data property
flags: [noStrict]
---*/

var o = {};
var desc = {
  value: 1,
  configurable: false,
}; // all other attributes default to false
Object.defineProperty(o, 'foo', desc);

// Now, deleting o.foo should fail because [[Configurable]] on foo is false.
var d = delete o.foo;

assert.sameValue(d, false, 'd');
assert.sameValue(o.hasOwnProperty('foo'), true, 'o.hasOwnProperty("foo")');
