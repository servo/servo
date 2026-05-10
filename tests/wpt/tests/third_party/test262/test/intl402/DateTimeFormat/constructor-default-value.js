// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Tests that the constructor for Intl.DateTimeFormat uses appropriate default
  values for its arguments (locales and options).
---*/

const actual = new Intl.DateTimeFormat().resolvedOptions();
const expected = new Intl.DateTimeFormat(
  [],
  Object.create(null)
).resolvedOptions();

assert.sameValue(actual.locale, expected.locale);
assert.sameValue(actual.calendar, expected.calendar);
assert.sameValue(actual.day, expected.day);
assert.sameValue(actual.month, expected.month);
assert.sameValue(actual.year, expected.year);
assert.sameValue(actual.numberingSystem, expected.numberingSystem);
assert.sameValue(actual.timeZone, expected.timeZone);
