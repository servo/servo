// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setmilliseconds
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setMilliseconds ( millisecond )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let millisecondNumber be ? ToNumber(millisecond).
  5. If tv is NaN, return NaN.
  6. Set tv to LocalTime(tv).
  7. Let time be MakeTime(𝔽(HourFromTime(tv)), 𝔽(MinuteFromTime(tv)), 𝔽(SecondFromTime(tv)), millisecondNumber).
  8. Let u be TimeClip(UTC(MakeDate(𝔽(Day(tv)), time))).
  9. Set dateObj.[[DateValue]] to u.
  10. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setMilliseconds.apply(date, args);

    assert.sameValue(result, NaN, "setMilliseconds(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setMilliseconds(" + args.join(", ") + ") [[DateValue]]");
  }
}
