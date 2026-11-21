// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setutcminutes
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setUTCMinutes ( minute [ , second [ , millisecond ] ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let minuteNumber be ? ToNumber(minute).
  5. If second is present, let secondNumber be ? ToNumber(second).
  6. If millisecond is present, let millisecondNumber be ? ToNumber(millisecond).
  7. If tv is NaN, return NaN.
  8. If second is not present, let secondNumber be 𝔽(SecondFromTime(tv)).
  9. If millisecond is not present, let millisecondNumber be 𝔽(MillisecondFromTime(tv)).
  10. Let date be MakeDate(𝔽(Day(tv)), MakeTime(𝔽(HourFromTime(tv)), minuteNumber, secondNumber, millisecondNumber)).
  11. Let v be TimeClip(date).
  12. Set dateObj.[[DateValue]] to v.
  13. Return v.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [5, 4, 3];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setUTCMinutes.apply(date, args);

    assert.sameValue(result, NaN, "setUTCMinutes(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setUTCMinutes(" + args.join(", ") + ") [[DateValue]]");
  }
}
