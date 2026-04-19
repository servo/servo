// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    create sets the [[Prototype]] of the created object to first parameter.
    This can be checked using isPrototypeOf, or getPrototypeOf.
es5id: 15.2.3.5-4-1
description: >
    Object.create sets the prototype of the passed-in object and adds
    new properties
---*/

function base() {}
var b = new base();
var prop = new Object();
var d = Object.create(b, {
  "x": {
    value: true,
    writable: false
  },
  "y": {
    value: "str",
    writable: false
  }
});

assert.sameValue(Object.getPrototypeOf(d), b, 'Object.getPrototypeOf(d)');
assert.sameValue(b.isPrototypeOf(d), true, 'b.isPrototypeOf(d)');
assert.sameValue(d.x, true, 'd.x');
assert.sameValue(d.y, "str", 'd.y');
assert.sameValue(b.x, undefined, 'b.x');
assert.sameValue(b.y, undefined, 'b.y');
