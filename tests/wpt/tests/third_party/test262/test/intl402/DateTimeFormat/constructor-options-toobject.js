// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Tests that Intl.DateTimeFormat contructor converts the options argument
  to an object using `ToObject` (7.1.13).

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

  const actual = new Intl.DateTimeFormat(['en-US'], value).resolvedOptions();
  const expected = new Intl.DateTimeFormat(['en-US'], result).resolvedOptions();

  assert.sameValue(actual.locale, expected.locale);
  assert.sameValue(actual.calendar, expected.calendar);
  assert.sameValue(actual.day, expected.day);
  assert.sameValue(actual.month, expected.month);
  assert.sameValue(actual.year, expected.year);
  assert.sameValue(actual.numberingSystem, expected.numberingSystem);
  assert.sameValue(actual.timeZone, expected.timeZone);
});

// ToObject throws a TypeError for undefined and null, but it's not called
// when options is undefined.
assert.throws(TypeError, () => new Intl.DateTimeFormat(['en-US'], null));
