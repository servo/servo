// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number-format-functions
description: >
  Tests that Intl.NumberFormat.prototype.format converts its argument
  (called value) to a number using ToNumber (7.1.3).
info: |
  11.1.4Number Format Functions

  4. Let x be ? ToNumber(value).
features: [Symbol]
---*/

const toNumberResults = [
  [undefined, NaN],
  [null, +0],
  [true, 1],
  [false, 0],
  ['42', 42],
  ['foo', NaN],
  ['Infinity', Infinity],
  ['-Infinity', -Infinity]
];

const nf = new Intl.NumberFormat();

toNumberResults.forEach(pair => {
  const [value, result] = pair;
  assert.sameValue(nf.format(value), nf.format(result));
});

let count = 0;
const dummy = {};
dummy[Symbol.toPrimitive] = hint => (hint === 'number' ? ++count : NaN);
assert.sameValue(nf.format(dummy), nf.format(count));
assert.sameValue(count, 1);

assert.throws(
  TypeError,
  () => nf.format(Symbol()),
  "ToNumber(arg) throws a TypeError when arg is of type 'Symbol'"
);
