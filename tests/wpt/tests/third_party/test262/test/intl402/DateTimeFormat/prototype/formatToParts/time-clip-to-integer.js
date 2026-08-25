// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: |
  TimeClip applies ToInteger on its input value.
info: >
  12.1.6 PartitionDateTimePattern ( dateTimeFormat, x )

  1. Let x be TimeClip(x).
  2. ...

  20.3.1.15 TimeClip ( time )
  ...
  3. Let clippedTime be ! ToInteger(time).
  4. If clippedTime is -0, set clippedTime to +0.
  5. Return clippedTime.
---*/

// Switch to a time format instead of using DateTimeFormat's default date-only format.
var dtf = new Intl.DateTimeFormat(undefined, {
    hour: "numeric", minute: "numeric", second: "numeric"
});

function formatAsString(dtf, time) {
    return dtf.formatToParts(time).map(part => part.value).join("");
}

var expected = formatAsString(dtf, 0);

assert.sameValue(formatAsString(dtf, -0.9), expected, "formatToParts(-0.9)");
assert.sameValue(formatAsString(dtf, -0.5), expected, "formatToParts(-0.5)");
assert.sameValue(formatAsString(dtf, -0.1), expected, "formatToParts(-0.1)");
assert.sameValue(formatAsString(dtf, -Number.MIN_VALUE), expected, "formatToParts(-Number.MIN_VALUE)");
assert.sameValue(formatAsString(dtf, -0), expected, "formatToParts(-0)");
assert.sameValue(formatAsString(dtf, +0), expected, "formatToParts(+0)");
assert.sameValue(formatAsString(dtf, Number.MIN_VALUE), expected, "formatToParts(Number.MIN_VALUE)");
assert.sameValue(formatAsString(dtf, 0.1), expected, "formatToParts(0.1)");
assert.sameValue(formatAsString(dtf, 0.5), expected, "formatToParts(0.5)");
assert.sameValue(formatAsString(dtf, 0.9), expected, "formatToParts(0.9)");
