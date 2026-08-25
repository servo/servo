// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: >
  Duration components are precise mathematical integers.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let duration = Temporal.Duration.from({
  seconds: Number.MAX_SAFE_INTEGER,
  nanoseconds: 999_999_999,
});

let time = new Temporal.PlainTime(0, 0, 0, 0, 0, 0);

let result = time.add(duration);

TemporalHelpers.assertPlainTime(result, 7, 36, 31, 999, 999, 999);
