// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: PlainDate.from enforces the supported range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const tooEarly = { year: -271821, month: 4, day: 18 };
const tooLate = { year: 275760, month: 9, day: 14 };
["reject", "constrain"].forEach((overflow) => {
  [tooEarly, tooLate, "-271821-04-18", "+275760-09-14"].forEach((value) => {
    assert.throws(RangeError, () => Temporal.PlainDate.from(value, { overflow }),
      `${JSON.stringify(value)} with ${overflow}`);
  });
});

TemporalHelpers.assertPlainDate(Temporal.PlainDate.from({ year: -271821, month: 4, day: 19 }),
  -271821, 4, "M04", 19, "min object");
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from({ year: 275760, month: 9, day: 13 }),
  275760, 9, "M09", 13, "max object");

TemporalHelpers.assertPlainDate(Temporal.PlainDate.from("-271821-04-19"),
  -271821, 4, "M04", 19, "min string");
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from("+275760-09-13"),
  275760, 9, "M09", 13, "max string");
