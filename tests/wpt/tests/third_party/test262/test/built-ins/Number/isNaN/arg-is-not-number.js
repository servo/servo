// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.isnan
description: >
  Return false if argument is not Number
info: |
  Number.isNaN ( number )

  1. If Type(number) is not Number, return false.
  [...]
features: [Symbol]
---*/

assert.sameValue(Number.isNaN("NaN"), false, "string");
assert.sameValue(Number.isNaN([NaN]), false, "[NaN]");
assert.sameValue(Number.isNaN(new Number(NaN)), false, "Number object");
assert.sameValue(Number.isNaN(false), false, "false");
assert.sameValue(Number.isNaN(true), false, "true");
assert.sameValue(Number.isNaN(undefined), false, "undefined");
assert.sameValue(Number.isNaN(null), false, "null");
assert.sameValue(Number.isNaN(Symbol("1")), false, "symbol");
assert.sameValue(Number.isNaN(), false, "no arg");
