// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-year-month-date-hours-minutes-seconds-ms
info: |
    When Date is called as a function rather than as a constructor,
    it should be "string" representing the current time (UTC)
es5id: 15.9.2.1_A1
description: Checking type of returned value
---*/
assert.sameValue(typeof Date(), "string", 'The value of `typeof Date()` is expected to be "string"');
assert.sameValue(typeof Date(1), "string", 'The value of `typeof Date(1)` is expected to be "string"');
assert.sameValue(typeof Date(1970, 1), "string", 'The value of `typeof Date(1970, 1)` is expected to be "string"');

assert.sameValue(
  typeof Date(1970, 1, 1),
  "string",
  'The value of `typeof Date(1970, 1, 1)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(1970, 1, 1, 1),
  "string",
  'The value of `typeof Date(1970, 1, 1, 1)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(1970, 1, 1, 1),
  "string",
  'The value of `typeof Date(1970, 1, 1, 1)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(1970, 1, 1, 1, 0),
  "string",
  'The value of `typeof Date(1970, 1, 1, 1, 0)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(1970, 1, 1, 1, 0, 0),
  "string",
  'The value of `typeof Date(1970, 1, 1, 1, 0, 0)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(1970, 1, 1, 1, 0, 0, 0),
  "string",
  'The value of `typeof Date(1970, 1, 1, 1, 0, 0, 0)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(Number.NaN),
  "string",
  'The value of `typeof Date(Number.NaN)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(Number.POSITIVE_INFINITY),
  "string",
  'The value of `typeof Date(Number.POSITIVE_INFINITY)` is expected to be "string"'
);

assert.sameValue(
  typeof Date(Number.NEGATIVE_INFINITY),
  "string",
  'The value of `typeof Date(Number.NEGATIVE_INFINITY)` is expected to be "string"'
);

assert.sameValue(typeof Date(undefined), "string", 'The value of `typeof Date(undefined)` is expected to be "string"');
assert.sameValue(typeof Date(null), "string", 'The value of `typeof Date(null)` is expected to be "string"');
