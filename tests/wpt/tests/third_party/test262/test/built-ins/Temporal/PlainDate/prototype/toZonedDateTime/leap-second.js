// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Leap second is a valid ISO string for PlainTime
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);

let arg = "2016-12-31T23:59:60";
const result1 = instance.toZonedDateTime({ plainTime: arg, timeZone: "UTC" });
assert.sameValue(
  result1.epochNanoseconds,
  957311999_000_000_000n,
  "leap second is a valid ISO string for PlainTime"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result2 = instance.toZonedDateTime({ plainTime: arg, timeZone: "UTC" });
assert.sameValue(
  result2.epochNanoseconds,
  957311999_000_000_000n,
  "second: 60 is ignored in property bag for PlainTime"
);
