// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setmonth
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setMonth ( month [ , day ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let monthNumber be ? ToNumber(month).
  5. If day is present, let dayNumber be ? ToNumber(day).
  6. If tv is NaN, return NaN.
  7. Set tv to LocalTime(tv).
  8. If day is not present, let dayNumber be 𝔽(DateFromTime(tv)).
  9. Let newDate be MakeDate(MakeDay(𝔽(YearFromTime(tv)), monthNumber, dayNumber), 𝔽(TimeWithinDay(tv))).
  10. Let u be TimeClip(UTC(newDate)).
  11. Set dateObj.[[DateValue]] to u.
  12. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [8, 7];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setMonth.apply(date, args);

    assert.sameValue(result, NaN, "setMonth(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setMonth(" + args.join(", ") + ") [[DateValue]]");
  }
}
