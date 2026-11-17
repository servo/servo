// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setminutes
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setMinutes ( minute [ , second [ , millisecond ] ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let minuteNumber be ? ToNumber(minute).
  5. If second is present, let secondNumber be ? ToNumber(second).
  6. If millisecond is present, let millisecondNumber be ? ToNumber(millisecond).
  7. If tv is NaN, return NaN.
  8. Set tv to LocalTime(tv).
  9. If second is not present, let secondNumber be 𝔽(SecondFromTime(tv)).
  10. If millisecond is not present, let millisecondNumber be 𝔽(MillisecondFromTime(tv)).
  11. Let date be MakeDate(𝔽(Day(tv)), MakeTime(𝔽(HourFromTime(tv)), minuteNumber, secondNumber, millisecondNumber)).
  12. Let u be TimeClip(UTC(date)).
  13. Set dateObj.[[DateValue]] to u.
  14. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [5, 4, 3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setMinutes.apply(date, args);

    assert.sameValue(result, NaN, "setMinutes(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setMinutes(" + args.join(", ") + ") [[DateValue]]");
  }
}
