// Copyright (C) 2018 Andrew Paprocki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.parse
description: >
  Date.parse return value is limited to specified time value maximum range
info: |
  Date.parse ( string )

  parse interprets the resulting String as a date and time; it returns a
  Number, the UTC time value corresponding to the date and time.

  A Date object contains a Number indicating a particular instant in time to
  within a millisecond. Such a Number is called a time value.

  The actual range of times supported by ECMAScript Date objects is slightly
  smaller: exactly -100,000,000 days to 100,000,000 days measured relative to
  midnight at the beginning of 01 January, 1970 UTC. This gives a range of
  8,640,000,000,000,000 milliseconds to either side of 01 January, 1970 UTC.
---*/

const minDateStr = "-271821-04-20T00:00:00.000Z";
const minDate = new Date(-8640000000000000);

assert.sameValue(minDate.toISOString(), minDateStr, "minDateStr");
assert.sameValue(Date.parse(minDateStr), minDate.valueOf(), "parse minDateStr");

const maxDateStr = "+275760-09-13T00:00:00.000Z";
const maxDate = new Date(8640000000000000);

assert.sameValue(maxDate.toISOString(), maxDateStr, "maxDateStr");
assert.sameValue(Date.parse(maxDateStr), maxDate.valueOf(), "parse maxDateStr");

const belowRange = "-271821-04-19T23:59:59.999Z";
const aboveRange = "+275760-09-13T00:00:00.001Z";

assert.sameValue(Date.parse(belowRange), NaN, "parse below minimum time value");
assert.sameValue(Date.parse(aboveRange), NaN, "parse above maximum time value");
