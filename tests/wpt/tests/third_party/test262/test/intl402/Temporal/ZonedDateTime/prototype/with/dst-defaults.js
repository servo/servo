// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Test behaviour around DST boundaries with the option offset set.
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dstStartDay = Temporal.PlainDateTime.from("2000-04-02T12:00:01").toZonedDateTime("America/Vancouver");
const dstEndDay = Temporal.PlainDateTime.from("2000-10-29T12:00:01").toZonedDateTime("America/Vancouver");
const twoThirty = {
  hour: 2,
  minute: 30
};

TemporalHelpers.assertZonedDateTimesEqual(
  dstStartDay.with(twoThirty),
  dstStartDay.with(twoThirty, { offset: "prefer" }),
  "Option offset: prefer is the default");

// disambiguation: compatible is the default
const zdt1 = dstStartDay.with(twoThirty, { offset: "ignore" });
const zdt2 = dstStartDay.with(twoThirty, { offset: "ignore", disambiguation: "compatible" });
assert.sameValue(
  zdt1.offset,
  zdt2.offset,
  "Option disambiguation: compatible is the default");
