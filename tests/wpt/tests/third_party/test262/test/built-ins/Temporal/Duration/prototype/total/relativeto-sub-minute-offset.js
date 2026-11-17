// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo string accepts trailing zeroes in sub-minute UTC offset
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

let result;
let relativeTo;

const action = (relativeTo) => instance.total({ unit: "days", relativeTo });

relativeTo = "1970-01-01T00:00-00:45:00[-00:45]";
result = action(relativeTo);
assert.sameValue(result, 366, "ISO string offset accepted with zero seconds (string)");

relativeTo = { year: 1970, month: 1, day: 1, offset: "+00:45:00.000000000", timeZone: "+00:45" };
result = action(relativeTo);
assert.sameValue(result, 366, "ISO string offset accepted with zero seconds (property bag)");

relativeTo = "1970-01-01T00:00+00:44:30.123456789[+00:45]";
assert.throws(RangeError, () => action(relativeTo), "rounding is not accepted between ISO offset and time zone");

relativeTo = "1970-01-01T00:00-00:44:59[-00:44:59]";
assert.throws(RangeError, () => action(relativeTo), "sub-minute offset not accepted as time zone identifier");

relativeTo = "1970-01-01T00:00-00:45:00[-00:45:00]";
assert.throws(RangeError, () => action(relativeTo), "offset with zero seconds not accepted as time zone identifier (ISO string)");

for (const timeZone of ["-00:44:59", "-004459", "+00:45:00", "+004500", "+15:59:00", "-170100"]) {
  relativeTo = { year: 1970, month: 1, day: 1, timeZone };
  assert.throws(RangeError, () => action(relativeTo), `${timeZone} is not a valid time zone string in property bag`);
}
