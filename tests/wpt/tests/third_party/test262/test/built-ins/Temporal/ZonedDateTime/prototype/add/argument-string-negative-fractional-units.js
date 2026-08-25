// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Strings with fractional duration units are treated with the correct sign
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

const resultHours = instance.add("-PT24.567890123H");
assert.sameValue(resultHours.epochNanoseconds, 999_911_555_595_557_200n, "negative fractional hours");

const resultMinutes = instance.add("-PT1440.567890123M");
assert.sameValue(resultMinutes.epochNanoseconds, 999_913_565_926_592_620n, "negative fractional minutes");
