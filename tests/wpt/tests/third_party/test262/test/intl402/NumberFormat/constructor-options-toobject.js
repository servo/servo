// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Tests that Intl.NumberFormat contructor converts the options argument
  to an object using `ToObject` (7.1.13).
info: |
  11.1.2 InitializeNumberFormat

  3.a. Let options be ? ToObject(options).
---*/

const toObjectResults = [
  [true, new Boolean(true)],
  [42, new Number(42)],
  ['foo', new String('foo')],
  [{}, {}],
  [Symbol(), Object(Symbol())]
];

// Test if ToObject is used to convert primitives to Objects.
toObjectResults.forEach(pair => {
  const [value, result] = pair;
  const actual = new Intl.NumberFormat(['en-US'], value).resolvedOptions();
  const expected = new Intl.NumberFormat(['en-US'], result).resolvedOptions();
  assert.sameValue(actual.locale, expected.locale);
  assert.sameValue(actual.minimumIntegerDigits, expected.minimumIntegerDigits);
  assert.sameValue(actual.minimumFractionDigits, expected.minimumFractionDigits);
  assert.sameValue(actual.maximumFractionDigits, expected.maximumFractionDigits);
  assert.sameValue(actual.numberingSystem, expected.numberingSystem);
  assert.sameValue(actual.style, expected.style);
  assert.sameValue(actual.useGrouping, expected.useGrouping);

});

// ToObject throws a TypeError for undefined and null, but it's not called
// when options is undefined.
assert.throws(TypeError, () => new Intl.NumberFormat(['en-US'], null));
