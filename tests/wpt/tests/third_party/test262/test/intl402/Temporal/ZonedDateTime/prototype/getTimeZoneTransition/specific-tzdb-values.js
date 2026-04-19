// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Smoke test specific values from time zone database
features: [Temporal]
---*/

const a1 = new Temporal.ZonedDateTime(1555448460_000_000_000n /* = 2019-04-16T21:01Z */, "America/New_York");
assert.sameValue(a1.getTimeZoneTransition("next").epochNanoseconds, 1572760800_000_000_000n /* = 2019-11-03T06:00:00Z */);

const a2 = new Temporal.ZonedDateTime(-5364662400_000_000_000n /* = 1800-01-01T00:00Z */, "America/New_York");
assert.sameValue(a2.getTimeZoneTransition("next").epochNanoseconds, -2717650800_000_000_000n /* = 1883-11-18T17:00:00Z */);

const a3 = new Temporal.ZonedDateTime(1591909260_000_000_000n /* = 2020-06-11T21:01Z */, "Europe/London");
assert.sameValue(a3.getTimeZoneTransition("previous").epochNanoseconds, 1585443600_000_000_000n /* = 2020-03-29T01:00:00Z */);

const a4 = new Temporal.ZonedDateTime(-3849984000_000_000_000n /* = 1848-01-01T00:00Z */, "Europe/London");
assert.sameValue(a4.getTimeZoneTransition("previous").epochNanoseconds, -3852662325_000_000_000n, /* = 1847-12-01T00:01:15Z */);
