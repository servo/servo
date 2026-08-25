// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
    Strict Mode - TypeError is thrown if The LeftHandSide of a Logical
    Assignment operator(??=) is a reference to a non-existent property
    of an object whose [[Extensible]] internal property is false.
flags: [onlyStrict]
features: [logical-assignment-operators]

---*/

var obj = {};
Object.preventExtensions(obj);

assert.throws(TypeError, function() {
  obj.prop ??= 1;
});
assert.sameValue(obj.prop, undefined, "obj.prop");
