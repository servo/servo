// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutchours
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setUTCHours ( hour [ , minute [ , second [ , millisecond ] ] ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let hourNumber be ? ToNumber(hour).
  5. If minute is present, let minuteNumber be ? ToNumber(minute).
  6. If second is present, let secondNumber be ? ToNumber(second).
  7. If millisecond is present, let millisecondNumber be ? ToNumber(millisecond).
  8. If tv is NaN, return NaN.
  9. If minute is not present, let minuteNumber be 𝔽(MinuteFromTime(tv)).
  10. If second is not present, let secondNumber be 𝔽(SecondFromTime(tv)).
  11. If millisecond is not present, let millisecondNumber be 𝔽(MillisecondFromTime(tv)).
  12. Let date be MakeDate(𝔽(Day(tv)), MakeTime(hourNumber, minuteNumber, secondNumber, millisecondNumber)).
  13. Let v be TimeClip(date).
  14. Set dateObj.[[DateValue]] to v.
  15. Return v.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [6, 5, 4, 3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setUTCHours.apply(date, args);

    assert.sameValue(result, NaN, "setUTCHours(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setUTCHours(" + args.join(", ") + ") [[DateValue]]");
  }
}
