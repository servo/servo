// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Throws if given a string.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

assert.throws(TypeError, () => zdt.with("1976-11-18T12:00+00:00[UTC]"));
assert.throws(TypeError, () => zdt.with("1976-11-18"));
assert.throws(TypeError, () => zdt.with("12:00"));
assert.throws(TypeError, () => zdt.with("invalid"));
