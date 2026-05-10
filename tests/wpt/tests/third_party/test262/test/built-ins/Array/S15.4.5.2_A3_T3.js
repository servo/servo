// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the length property is changed, every property whose name
    is an array index whose value is not smaller than the new length is automatically deleted
es5id: 15.4.5.2_A3_T3
description: "[[Put]] (length, 4294967296)"
---*/

var x = [];
x.length = 4294967295;
assert.sameValue(x.length, 4294967295, 'The value of x.length is expected to be 4294967295');

try {
  x = [];
  x.length = 4294967296;
  throw new Test262Error('#2.1: x = []; x.length = 4294967296 throw RangeError. Actual: x.length === ' + (x.length));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}
