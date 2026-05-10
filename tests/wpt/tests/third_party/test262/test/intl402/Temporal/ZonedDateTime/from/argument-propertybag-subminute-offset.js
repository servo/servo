// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test a subminute timezone when the argument is a property bag.
features: [Temporal]
---*/

// Does not truncate offset property to minutes
const zdt = Temporal.ZonedDateTime.from({
  year: 1971,
  month: 1,
  day: 1,
  hour: 12,
  timeZone: "Africa/Monrovia"
});
assert.sameValue(zdt.offset, "-00:44:30");
