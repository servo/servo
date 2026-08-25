// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-year-month-date-hours-minutes-seconds-ms
info: |
    All of the arguments are optional, any arguments supplied are
    accepted but are completely ignored. A string is created and returned as
    if by the expression (new Date()).toString()
es5id: 15.9.2.1_A2
description: Use various number arguments and various types of ones
---*/

function isEqual(d1, d2) {
  if (d1 === d2) {
    return true;
  } else if (Math.abs(Date.parse(d1) - Date.parse(d2)) <= 1000) {
    return true;
  } else {
    return false;
  }
}

assert(
  isEqual(Date(), (new Date()).toString()),
  'isEqual(Date(), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1), (new Date()).toString()),
  'isEqual(Date(1), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1), (new Date()).toString()),
  'isEqual(Date(1970, 1), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1, 1), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1, 1), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1, 1), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1, 1), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1, 1, 0), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1, 1, 0), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1, 1, 0, 0), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1, 1, 0, 0), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(1970, 1, 1, 1, 0, 0, 0), (new Date()).toString()),
  'isEqual(Date(1970, 1, 1, 1, 0, 0, 0), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(Number.NaN), (new Date()).toString()),
  'isEqual(Date(Number.NaN), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(Number.POSITIVE_INFINITY), (new Date()).toString()),
  'isEqual(Date(Number.POSITIVE_INFINITY), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(Number.NEGATIVE_INFINITY), (new Date()).toString()),
  'isEqual(Date(Number.NEGATIVE_INFINITY), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(undefined), (new Date()).toString()),
  'isEqual(Date(undefined), (new Date()).toString()) must return true'
);

assert(
  isEqual(Date(null), (new Date()).toString()),
  'isEqual(Date(null), (new Date()).toString()) must return true'
);
