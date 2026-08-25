// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Next and previous transition in a named time zone has the correct return type
features: [Temporal]
---*/

let zdt = new Temporal.ZonedDateTime(0n, "America/Los_Angeles");
for (let count = 0; count < 4; count++) {
  const transition = zdt.getTimeZoneTransition("next");
  assert(transition instanceof Temporal.ZonedDateTime, "getTimeZoneTransition(next) returns Temporal.ZonedDateTime");
  assert(!transition.equals(zdt), "getTimeZoneTransition(next) does not return its input");
  zdt = transition;
}

zdt = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "America/Los_Angeles");
for (let count = 0; count < 4; count++) {
  const transition = zdt.getTimeZoneTransition("previous");
  assert(transition instanceof Temporal.ZonedDateTime, "getTimeZoneTransition(previous) returns Temporal.ZonedDateTime");
  assert(!transition.equals(zdt), "getTimeZoneTransition(previous) does not return its input");
  zdt = transition;
}
