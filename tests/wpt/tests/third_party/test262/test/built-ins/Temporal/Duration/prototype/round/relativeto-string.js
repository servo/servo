// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: relativeTo option accepts ISO date-time strings as argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

[
  ["2019-11-01T00:00", "bare date-time string is a plain relativeTo"],
  ["2019-11-01T00:00-07:00", "date-time + offset is a plain relativeTo"],
  ["2019-11-01T00:00[-07:00]", "date-time + IANA annotation is a zoned relativeTo"],
  ["2019-11-01T00:00Z[-07:00]", "date-time + Z + IANA annotation is a zoned relativeTo"],
  ["2019-11-01T00:00+00:00[UTC]", "date-time + offset + IANA annotation is a zoned relativeTo"],
  ['2000-01-01', "date"],
  ['20200101', "date without separators"],
  ["2000-01-01[UTC]", "date with timezone"],
  ["2000-01-01T00:00[UTC]","datetime with timezone"],
  ["2000-01-01T00:00+00:00[UTC]", "datetime with timezone and matching offset"],
  ["2000-01-01T00:00+00:00[UTC][u-ca=iso8601]", "datetime with timezone, offset, and calendar"],
].forEach(([relativeTo, description]) => {
  let result = instance.round({ largestUnit: "years", relativeTo });
  TemporalHelpers.assertDuration(result, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, description);
});

