// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Named time zones that no longer observe DST return null for the "next"
  direction when queried from a date after their last transition.
info: |
  Some time zones historically observed DST but no longer do. When queried for
  "next" from a date after the last historical transition, the result should be
  null. When queried for "previous", it should find the last historical
  transition.
features: [Temporal]
---*/

// Asia/Kolkata has not observed DST since 1945.
// Querying from 2024 should find no next transition but should find a previous one.
var zdt = Temporal.ZonedDateTime.from("2024-06-15T12:00:00[Asia/Kolkata]");

assert.sameValue(
  zdt.getTimeZoneTransition("next"),
  null,
  "Asia/Kolkata should have no next time zone transition from 2024"
);

var prev = zdt.getTimeZoneTransition("previous");
assert.notSameValue(
  prev,
  null,
  "Asia/Kolkata should have a previous time zone transition (historical)"
);
