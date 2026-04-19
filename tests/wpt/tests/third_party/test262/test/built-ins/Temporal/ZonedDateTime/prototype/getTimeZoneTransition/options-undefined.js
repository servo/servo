// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Options argument is required
info: |
  1. If _directionParam_ is *undefined*, throw a *TypeError* exception.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

assert.throws(TypeError, () => zdt.getTimeZoneTransition());
assert.throws(TypeError, () => zdt.getTimeZoneTransition(undefined));
