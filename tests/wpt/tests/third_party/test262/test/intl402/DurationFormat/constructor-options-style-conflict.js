// Copyright 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isvaliddurationrecord
description: Checks that the "long", "short", and "narrow" styles can't be used for units following a unit using the "numeric" or "2-digit" style.
info: |
    GetDurationUnitOptions (unit, options, baseStyle, stylesList, digitalBase, prevStyle)
    (...)
    6. If prevStyle is "numeric" or "2-digit", then
    a. If style is not "numeric" or "2-digit", then
        i. Throw a RangeError exception.
features: [Intl.DurationFormat]
---*/

let invalidOptions = {};
for (const timeUnit of ["hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"]){
  invalidOptions[timeUnit] = "numeric";
}

for (const timeUnit of ["minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"]){
  for (const invalidStyle of ["long", "short", "narrow"]){
    invalidOptions[timeUnit] = invalidStyle;
    assert.throws(RangeError, function() {
      new Intl.DurationFormat([], invalidOptions);
    }, `${invalidStyle} is an invalid style option value when following a unit with \"numeric\" style`);
  }
  invalidOptions[timeUnit] = "numeric";
}

for (const timeUnit of ["hours", "minutes", "seconds"]){
  invalidOptions[timeUnit] = "2-digit";
}

for (const timeUnit of ["minutes", "seconds", "milliseconds"]){
  for (const invalidStyle of ["long", "short", "narrow"]){
    invalidOptions[timeUnit] = invalidStyle;
    assert.throws(RangeError, function() {
      new Intl.DurationFormat([], invalidOptions);
    }, `${invalidStyle} is an invalid style option value when following a unit with \"2-digit\" style`);
  }
  invalidOptions[timeUnit] = "2-digit";
}
