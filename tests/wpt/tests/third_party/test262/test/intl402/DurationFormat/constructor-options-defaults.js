// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks handling of valid options for the DurationFormat constructor.
info: |
    Intl.DurationFormat  ( [ locales [ , options ] ] )
    (...)
    17. For each row in Table 1, except the header row, in table order, do
      a. Let styleSlot be the Style Slot value.
      b. Let displaySlot be the Display Slot value.
      c. Let unit be the Unit value.
      d. Let valueList be the Values value.
      e. Let digitalBase be the Digital Default value.
      f. Let unitOptions be ? GetUnitOptions(unit, options, style, valueList, digitalBase, prevStyle).
      g. Set durationFormat.[[<styleSlot>]] to unitOptions.[[Style]].
      h. Set durationFormat.[[<displaySlot>]] to unitOptions.[[Display]].
features: [Intl.DurationFormat]
includes: [testIntl.js]
---*/

testOption( Intl.DurationFormat, "years", "string", ["long", "short", "narrow"], "short");
testOption( Intl.DurationFormat, "months", "string", ["long", "short", "narrow"], "short");
testOption( Intl.DurationFormat, "weeks", "string", ["long", "short", "narrow"], "short");
testOption( Intl.DurationFormat, "days", "string", ["long", "short", "narrow"], "short");
testOption( Intl.DurationFormat, "hours", "string", ["long", "short", "narrow", "numeric", "2-digit"], "short");
testOption( Intl.DurationFormat, "minutes", "string", ["long", "short", "narrow", "numeric", "2-digit"], "short");
testOption( Intl.DurationFormat, "seconds", "string", ["long", "short", "narrow", "numeric", "2-digit"], "short");
testOption( Intl.DurationFormat, "milliseconds", "string", ["long", "short", "narrow", "numeric"], "short");
testOption( Intl.DurationFormat, "microseconds", "string", ["long", "short", "narrow", "numeric"], "short");
testOption( Intl.DurationFormat, "nanoseconds", "string", ["long", "short", "narrow", "numeric"], "short");

