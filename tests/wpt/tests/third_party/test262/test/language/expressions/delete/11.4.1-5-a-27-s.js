// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    Strict Mode - TypeError is thrown after deleting a property,
    calling preventExtensions, and attempting to reassign the property
flags: [onlyStrict]
---*/

var a = {
  x: 0,
  get y() {
    return 0;
  },
};
delete a.x;
Object.preventExtensions(a);
assert.throws(TypeError, function() {
  a.x = 1;
});
