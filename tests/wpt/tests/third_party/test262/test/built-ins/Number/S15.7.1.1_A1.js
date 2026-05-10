// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number([value]) returns a number value (not a Number object) computed by
    ToNumber(value) if value was supplied
es5id: 15.7.1.1_A1
description: Used values "10", 10, new String("10"), new Object(10) and "abc"
---*/
assert.sameValue(typeof Number("10"), "number", 'The value of `typeof Number("10")` is expected to be "number"');
assert.sameValue(typeof Number(10), "number", 'The value of `typeof Number(10)` is expected to be "number"');

assert.sameValue(
  typeof Number(new String("10")),
  "number",
  'The value of `typeof Number(new String("10"))` is expected to be "number"'
);

assert.sameValue(
  typeof Number(new Object(10)),
  "number",
  'The value of `typeof Number(new Object(10))` is expected to be "number"'
);

assert.sameValue(Number("abc"), NaN, 'Number("abc") returns NaN');
assert.sameValue(Number("INFINITY"), NaN, 'Number("INFINITY") returns NaN');
assert.sameValue(Number("infinity"), NaN, 'Number("infinity") returns NaN');
