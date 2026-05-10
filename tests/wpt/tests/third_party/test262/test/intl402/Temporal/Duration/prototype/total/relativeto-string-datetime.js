// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
    Conversion of ISO date-time strings as relativeTo option to
    Temporal.ZonedDateTime or Temporal.PlainDateTime instances
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

let relativeTo = "2019-11-01T00:00[America/Vancouver]";
const result4 = instance.total({ unit: "days", relativeTo });
assert.sameValue(result4, 366.96, "date-time + IANA annotation is a zoned relativeTo");

relativeTo = "2019-11-01T00:00Z[America/Vancouver]";
const result5 = instance.total({ unit: "days", relativeTo });
assert.sameValue(result5, 366.96, "date-time + Z + IANA annotation is a zoned relativeTo");

relativeTo = "2019-11-01T00:00-07:00[America/Vancouver]";
const result6 = instance.total({ unit: "days", relativeTo });
assert.sameValue(result6, 366.96, "date-time + offset + IANA annotation is a zoned relativeTo");

relativeTo = "2019-11-01T00:00+04:15[America/Vancouver]";
assert.throws(RangeError, () => instance.total({ unit: "days", relativeTo }), "date-time + offset + IANA annotation throws if wall time and exact time mismatch");
