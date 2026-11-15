// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setseconds
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setSeconds ( second [ , millisecond ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let secondNumber be ? ToNumber(second).
  5. If millisecond is present, let millisecondNumber be ? ToNumber(millisecond).
  6. If tv is NaN, return NaN.
  7. Set tv to LocalTime(tv).
  8. If millisecond is not present, let millisecondNumber be 𝔽(MillisecondFromTime(tv)).
  9. Let date be MakeDate(𝔽(Day(tv)), MakeTime(𝔽(HourFromTime(tv)), 𝔽(MinuteFromTime(tv)), secondNumber, millisecondNumber)).
  10. Let u be TimeClip(UTC(date)).
  11. Set dateObj.[[DateValue]] to u.
  12. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [4, 3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setSeconds.apply(date, args);

    assert.sameValue(result, NaN, "setSeconds(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setSeconds(" + args.join(", ") + ") [[DateValue]]");
  }
}
