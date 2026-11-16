// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.settime
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setTime ( time )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let t be ? ToNumber(time).
  4. Let v be TimeClip(t).
  5. Set dateObj.[[DateValue]] to v.
  6. Return v.

  TimeClip ( time )

  1. If time is not a finite Number, return NaN.
---*/

var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < nonFinite.length; i++) {
  var date = new Date(2016, 6, 1);

  var result = date.setTime(nonFinite[i]);

  assert.sameValue(result, NaN, "setTime(" + nonFinite[i] + ") return value");
  assert.sameValue(date.getTime(), NaN, "setTime(" + nonFinite[i] + ") [[DateValue]]");
}
