// Copyright (C) 2026 Hanzhi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setfullyear
description: >
  A non-finite argument on a valid date sets [[DateValue]] to NaN
info: |
  Date.prototype.setFullYear ( year [ , month [ , day ] ] )

  1. Let dateObj be the this value.
  2. Perform ? RequireInternalSlot(dateObj, [[DateValue]]).
  3. Let tv be dateObj.[[DateValue]].
  4. Let yearNumber be ? ToNumber(year).
  5. If tv is NaN, set tv to +0𝔽; else set tv to LocalTime(tv).
  6. If month is present, let monthNumber be ? ToNumber(month); else let monthNumber be 𝔽(MonthFromTime(tv)).
  7. If day is present, let dayNumber be ? ToNumber(day); else let dayNumber be 𝔽(DateFromTime(tv)).
  8. Let newDate be MakeDate(MakeDay(yearNumber, monthNumber, dayNumber), 𝔽(TimeWithinDay(tv))).
  9. Let u be TimeClip(UTC(newDate)).
  10. Set dateObj.[[DateValue]] to u.
  11. Return u.

  MakeTime, MakeDay and MakeDate return NaN when an argument is not finite.
---*/

var validArgs = [2017, 8, 7];
var nonFinite = [NaN, Infinity, -Infinity];

for (var i = 0; i < validArgs.length; i++) {
  for (var j = 0; j < nonFinite.length; j++) {
    var date = new Date(2016, 6, 1);
    var args = validArgs.slice(0, i + 1);
    args[i] = nonFinite[j];

    var result = date.setFullYear.apply(date, args);

    assert.sameValue(result, NaN, "setFullYear(" + args.join(", ") + ") return value");
    assert.sameValue(date.getTime(), NaN, "setFullYear(" + args.join(", ") + ") [[DateValue]]");
  }
}
