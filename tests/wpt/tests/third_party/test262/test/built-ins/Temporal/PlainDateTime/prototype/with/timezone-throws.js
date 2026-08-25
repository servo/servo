// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if a timezone is supplied
esid: sec-temporal.plaindatetime.prototype.with
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

assert.throws(
  TypeError,
  () => datetime.with({ year: 2021, timeZone: "UTC" }),
  "throws with timezone property"
);
