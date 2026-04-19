// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: >
  Tests that Intl.NumberFormat.prototype.formatToParts converts
  its argument (called value) to a number using ToNumber (7.1.3).
info: |
  11.1.4 Number Format Functions

  4. Let x be ? ToNumber(value).
features: [Symbol]
---*/

const toNumberResults = [
  [undefined, NaN],
  [null, +0],
  [true, 1],
  [false, +0],
  ['42', 42],
  ['foo', NaN]
];

const nf = new Intl.NumberFormat();

function assertSameParts(actual, expected) {
  assert.sameValue(actual.length, expected.length);
  for (let i = 0; i < expected.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type);
    assert.sameValue(actual[i].value, expected[i].value);
  }
}

toNumberResults.forEach(pair => {
  const [value, result] = pair;
  assertSameParts(nf.formatToParts(value), nf.formatToParts(result));
});

let count = 0;
const dummy = {};
dummy[Symbol.toPrimitive] = hint => (hint === 'number' ? ++count : NaN);
assertSameParts(nf.formatToParts(dummy), nf.formatToParts(count));
assert.sameValue(count, 1);

assert.throws(
  TypeError,
  () => nf.formatToParts(Symbol()),
  "ToNumber(arg) throws a TypeError when arg is of type 'Symbol'"
);
