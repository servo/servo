// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: An offset time zone has no transitions.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "-10:00");
assert.sameValue(zdt.getTimeZoneTransition("next"), null, "An offset time zone has no next transition");
assert.sameValue(zdt.getTimeZoneTransition("previous"), null, "An offset time zone has no previous transition");
