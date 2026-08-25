// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Throws if given a Temporal.ZonedDateTime with a time zone
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

assert.throws(TypeError, () => zdt.with(zdt));
