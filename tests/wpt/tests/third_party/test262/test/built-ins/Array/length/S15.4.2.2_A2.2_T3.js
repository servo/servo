// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
info: |
    If the argument len is a Number and ToUint32(len) is not equal to len,
    a RangeError exception is thrown
es5id: 15.4.2.2_A2.2_T3
description: Use try statement. len = 1.5, Number.MAX_VALUE, Number.MIN_VALUE
---*/

try {
  new Array(1.5);
  throw new Test262Error('#1.1: new Array(1.5) throw RangeError. Actual: ' + (new Array(1.5)));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

try {
  new Array(Number.MAX_VALUE);
  throw new Test262Error('#2.1: new Array(Number.MAX_VALUE) throw RangeError. Actual: ' + (new Array(Number.MAX_VALUE)));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}

try {
  new Array(Number.MIN_VALUE);
  throw new Test262Error('#3.1: new Array(Number.MIN_VALUE) throw RangeError. Actual: ' + (new Array(Number.MIN_VALUE)));
} catch (e) {
  assert.sameValue(
    e instanceof RangeError,
    true,
    'The result of evaluating (e instanceof RangeError) is expected to be true'
  );
}
