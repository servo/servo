// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo option accepts ISO date-time strings as argument
features: [Temporal]
---*/

const instance1 = new Temporal.Duration(0, 0, 0, 31);
const instance2 = new Temporal.Duration(0, 1);

[
  ["2019-11-01T00:00", 1, "bare datetime string is a plain relativeTo"],
  ["2019-11-01T00:00-07:00", 1, "datetime + offset is a plain relativeTo"],
  ["2019-11-01T00:00[-07:00]", 1,  "datetime + IANA annotation is a zoned relativeTo"],
  ["2019-11-01T00:00Z[-07:00]", 1, "datetime + Z + IANA annotation is a zoned relativeTo"],
  ["2019-11-01T00:00+00:00[UTC]", 1, "datetime + offset + IANA annotation is a zoned relativeTo"],
  ['2000-01-01', 0, "date"],
  ['20200101', 0, "date without separators"],
  ["2000-01-01[UTC]", 0, "date with timezone"],
  ["2000-01-01T00:00[UTC]", 0, "datetime with timezone"],
  ["2000-01-01T00:00+00:00[UTC]", 0, "datetime with timezone and matching offset"],
  ["2000-01-01T00:00+00:00[UTC][u-ca=iso8601]", 0, "datetime with timezone, offset, and calendar"],
].forEach(([relativeTo, expected, description]) => {
  let result = Temporal.Duration.compare(instance1, instance2, { relativeTo });
  assert.sameValue(result, expected, description);
});
