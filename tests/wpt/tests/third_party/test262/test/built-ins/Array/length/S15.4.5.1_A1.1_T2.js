// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-exotic-objects-defineownproperty-p-desc
info: If ToUint32(length) !== ToNumber(length), throw RangeError
es5id: 15.4.5.1_A1.1_T2
description: length in [NaN, Infinity, -Infinity, undefined]
---*/

try {
  var x = [];
  x.length = NaN;
  throw new Test262Error('#1.1: x = []; x.length = NaN throw RangeError. Actual: x.length === ' + (x.length));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

try {
  x = [];
  x.length = Number.POSITIVE_INFINITY;
  throw new Test262Error('#2.1: x = []; x.length = Number.POSITIVE_INFINITY throw RangeError. Actual: x.length === ' + (x.length));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

try {
  x = [];
  x.length = Number.NEGATIVE_INFINITY;
  throw new Test262Error('#3.1: x = []; x.length = Number.NEGATIVE_INFINITY throw RangeError. Actual: x.length === ' + (x.length));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

try {
  x = [];
  x.length = undefined;
  throw new Test262Error('#4.1: x = []; x.length = undefined throw RangeError. Actual: x.length === ' + (x.length));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}
