// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for Array object
esid: sec-array.prototype.push
description: If ToUint32(length) !== length, throw RangeError
---*/

var x = [];
x.length = 4294967295;

var push = x.push();
assert.sameValue(push, 4294967295, 'The value of push is expected to be 4294967295');

try {
  x.push("x");
  throw new Test262Error('#2.1: x = []; x.length = 4294967295; x.push("x") throw RangeError. Actual: ' + (push));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

assert.sameValue(x[4294967295], "x", 'The value of x[4294967295] is expected to be "x"');
assert.sameValue(x.length, 4294967295, 'The value of x.length is expected to be 4294967295');
