// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: With offset 'reject', throws if offset does not match IANA time zone
features: [Temporal]
---*/

const obj = {
  year: 2020,
  month: 3,
  day: 8,
  hour: 1,
  offset: "-04:00",
  timeZone: "UTC"
};

assert.throws(RangeError, () => Temporal.ZonedDateTime.from(obj));
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(obj, { offset: "reject" }));
assert.throws(RangeError, () => Temporal.ZonedDateTime.from("2020-03-08T01:00-04:00[UTC]"));
assert.throws(RangeError, () => Temporal.ZonedDateTime.from("2020-03-08T01:00-04:00[UTC]", { offset: "reject" }));
