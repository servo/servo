// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
    Strict Mode - TypeError is not thrown if the LeftHandSide of a Logical
    Assignment operator(||=) is a reference to a data property with the
    attribute value {[[Writable]]:false} and PutValue step is not reached.
flags: [onlyStrict]
features: [logical-assignment-operators]

---*/

var obj = {};
Object.defineProperty(obj, "prop", {
  value: 2,
  writable: false,
  enumerable: true,
  configurable: true
});

assert.sameValue(obj.prop ||= 1, 2, "obj.prop");
