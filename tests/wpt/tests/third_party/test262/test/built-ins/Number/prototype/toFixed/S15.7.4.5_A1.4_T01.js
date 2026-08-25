// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Step 9: If x >= 10^21, let m = ToString(x)"
es5id: 15.7.4.5_A1.4_T01
description: x is 10^21
---*/
assert.sameValue(
  (new Number(1e21)).toFixed(),
  String(1e21),
  '(new Number(1e21)).toFixed() must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed(0),
  String(1e21),
  '(new Number(1e21)).toFixed(0) must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed(1),
  String(1e21),
  '(new Number(1e21)).toFixed(1) must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed(1.1),
  String(1e21),
  '(new Number(1e21)).toFixed(1.1) must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed(0.9),
  String(1e21),
  '(new Number(1e21)).toFixed(0.9) must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed("1"),
  String(1e21),
  '(new Number(1e21)).toFixed("1") must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed("1.1"),
  String(1e21),
  '(new Number(1e21)).toFixed("1.1") must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed("0.9"),
  String(1e21),
  '(new Number(1e21)).toFixed("0.9") must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed(Number.NaN),
  String(1e21),
  '(new Number(1e21)).toFixed(Number.NaN) must return the same value returned by String(1e21)'
);

assert.sameValue(
  (new Number(1e21)).toFixed("some string"),
  String(1e21),
  '(new Number(1e21)).toFixed("some string") must return the same value returned by String(1e21)'
);

try {
  s = (new Number(1e21)).toFixed(Number.POSITIVE_INFINITY);
  throw new Test262Error('#10: (new Number(1e21)).toFixed(Number.POSITIVE_INFINITY) should throw RangeError, not return NaN');
}
catch (e) {
  assert(e instanceof RangeError, 'The result of evaluating (e instanceof RangeError) is expected to be true');
}
