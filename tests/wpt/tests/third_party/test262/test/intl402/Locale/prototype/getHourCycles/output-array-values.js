// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getHourCycles
description: >
    Checks that the return value of Intl.Locale.prototype.getHourCycles is an Array
    that only contains valid values.
info: |
  HourCyclesOfLocale ( loc )
  ...
  4. Let list be a List of 1 or more unique hour cycle identifiers, which must
  be lower case String values indicating either the 12-hour format ("h11",
  "h12") or the 24-hour format ("h23", "h24"), sorted in descending preference
  of those in common use for date and time formatting in locale.
features: [Intl.Locale, Intl.Locale-info, Array.prototype.includes]
---*/

const output = new Intl.Locale('en').getHourCycles();
assert(output.length > 0, 'array has at least one element');
output.forEach(hc => {
  if(!['h11', 'h12', 'h23', 'h24'].includes(hc))
    throw new Test262Error();
});
