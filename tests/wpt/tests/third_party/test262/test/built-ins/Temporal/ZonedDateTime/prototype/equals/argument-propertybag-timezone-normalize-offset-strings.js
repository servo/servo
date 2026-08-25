// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Offset time zone identifiers are compared using their normal form, ignoring syntax differences in offset strings
features: [Temporal]
---*/

const tests = [
  { idToTest: "+0000", description: "colon-less" },
  { idToTest: "+00", description: "hours-only" }
];

for (const test of tests) {
  const {idToTest, description} = test;
  const instance = new Temporal.ZonedDateTime(0n, "+00:00");

  const bag = { year: 1970, monthCode: "M01", day: 1, timeZone: idToTest };
  assert.sameValue(instance.equals(bag), true, `Offset time zones are equal despite ${description} syntax in property bag argument`);
  
  const str = "1970-01-01[+00:00]";
  assert.sameValue(instance.equals(str), true, `Offset time zones are equal despite ${description} syntax in ISO string argument`);
}
