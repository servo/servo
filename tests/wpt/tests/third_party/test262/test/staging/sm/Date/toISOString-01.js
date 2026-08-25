// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Date.prototype.toISOString returns an invalid ISO-8601 string
info: bugzilla.mozilla.org/show_bug.cgi?id=730831
esid: pending
---*/

/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

function iso(t) {
  return new Date(t).toISOString();
}

function utc(year, month, day, hour, minute, second, millis) {
  var date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  date.setUTCHours(hour, minute, second, millis);
  return date.getTime();
}


// Values around maximum date for simple iso format.
var maxDateSimple = utc(9999, 12, 31, 23, 59, 59, 999);
assert.sameValue(iso(maxDateSimple - 1),    "9999-12-31T23:59:59.998Z");
assert.sameValue(iso(maxDateSimple    ),    "9999-12-31T23:59:59.999Z");
assert.sameValue(iso(maxDateSimple + 1), "+010000-01-01T00:00:00.000Z");


// Values around minimum date for simple iso format.
var minDateSimple = utc(0, 1, 1, 0, 0, 0, 0);
assert.sameValue(iso(minDateSimple - 1), "-000001-12-31T23:59:59.999Z");
assert.sameValue(iso(minDateSimple    ),    "0000-01-01T00:00:00.000Z");
assert.sameValue(iso(minDateSimple + 1),    "0000-01-01T00:00:00.001Z");


// Values around maximum date for extended iso format.
var maxDateExtended = utc(+275760, 9, 13, 0, 0, 0, 0);
assert.sameValue(maxDateExtended, +8.64e15);
assert.sameValue(iso(maxDateExtended - 1), "+275760-09-12T23:59:59.999Z");
assert.sameValue(iso(maxDateExtended    ), "+275760-09-13T00:00:00.000Z");
assert.throws(RangeError, () => iso(maxDateExtended + 1));


// Values around minimum date for extended iso format.
var minDateExtended = utc(-271821, 4, 20, 0, 0, 0, 0);
assert.sameValue(minDateExtended, -8.64e15);
assert.throws(RangeError, () => iso(minDateExtended - 1));
assert.sameValue(iso(minDateExtended    ), "-271821-04-20T00:00:00.000Z");
assert.sameValue(iso(minDateExtended + 1), "-271821-04-20T00:00:00.001Z");
