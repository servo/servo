// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.sethours
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setHours ( hour [ , minute [ , second [ , millisecond ] ] ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let hourNumber be ? ToNumber(hour).
  5. If minute is present, let minuteNumber be ? ToNumber(minute).
  6. If second is present, let secondNumber be ? ToNumber(second).
  7. If millisecond is present, let millisecondNumber be ? ToNumber(millisecond).
  8. If tv is NaN, return NaN.
  9. Set tv to LocalTime(tv).
  10. If minute is not present, let minuteNumber be 𝔽(MinuteFromTime(tv)).
  11. If second is not present, let secondNumber be 𝔽(SecondFromTime(tv)).
  12. If millisecond is not present, let millisecondNumber be 𝔽(MillisecondFromTime(tv)).
  13. Let date be MakeDate(𝔽(Day(tv)), MakeTime(hourNumber, minuteNumber, secondNumber, millisecondNumber)).
  14. Let u be TimeClip(UTC(date)).
  15. Set dateObj.[[DateValue]] to u.
  16. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [6, 5, 4, 3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setHours.apply(date, args);

    assert.sameValue(result, NaN, "setHours(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setHours(" + args.join(", ") + ") [[DateValue]]");
  }
}
