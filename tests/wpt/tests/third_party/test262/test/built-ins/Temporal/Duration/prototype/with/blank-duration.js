// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const blank = new Temporal.Duration();

for (const val of [-1, 0, 1]) {
  let result = blank.with({ years: val });
  TemporalHelpers.assertDuration(result, val, 0, 0, 0, 0, 0, 0, 0, 0, 0, `with years ${val}`);
  result = blank.with({ months: val });
  TemporalHelpers.assertDuration(result, 0, val, 0, 0, 0, 0, 0, 0, 0, 0, `with months ${val}`);
  result = blank.with({ weeks: val });
  TemporalHelpers.assertDuration(result, 0, 0, val, 0, 0, 0, 0, 0, 0, 0, `with weeks ${val}`);
  result = blank.with({ days: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, val, 0, 0, 0, 0, 0, 0, `with days ${val}`);
  result = blank.with({ hours: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, val, 0, 0, 0, 0, 0, `with hours ${val}`);
  result = blank.with({ minutes: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, val, 0, 0, 0, 0, `with minutes ${val}`);
  result = blank.with({ seconds: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, val, 0, 0, 0, `with seconds ${val}`);
  result = blank.with({ milliseconds: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, val, 0, 0, `with milliseconds ${val}`);
  result = blank.with({ microseconds: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, val, 0, `with microseconds ${val}`);
  result = blank.with({ nanoseconds: val });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, val, `with nanoseconds ${val}`);
}
