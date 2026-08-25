// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Throws if timeZone is included.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

// throws if timeZone is included
assert.throws(TypeError, () => zdt.with({
  month: 2,
  timeZone: "UTC"
}));
