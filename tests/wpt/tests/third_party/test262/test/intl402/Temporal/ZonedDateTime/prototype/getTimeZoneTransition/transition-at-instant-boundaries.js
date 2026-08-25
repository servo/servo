// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Test transitions at the instant boundaries.
features: [Temporal, Intl-enumeration]
---*/

for (let id of Intl.supportedValuesOf("timeZone")) {
  const min = new Temporal.ZonedDateTime(-86_40000_00000_00000_00000n, id);
  const max = new Temporal.ZonedDateTime(86_40000_00000_00000_00000n, id);

  const next = min.getTimeZoneTransition("next");
  if (next) {
    assert(next.epochNanoseconds > min.epochNanoseconds,
      "If there's any next transition, it should be after |min|");
  }

  const prev = max.getTimeZoneTransition("previous");
  if (prev) {
    assert(prev.epochNanoseconds < max.epochNanoseconds,
      "If there's any previous transition, it should be before |max|");
  }

  assert.sameValue(max.getTimeZoneTransition("next"), null, "There shouldn't be any next transition after |max|");
  assert.sameValue(min.getTimeZoneTransition("previous"), null,
    "There shouldn't be any previous transition before |min|");
}
