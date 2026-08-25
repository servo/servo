// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Named time zones without DST return null for getTimeZoneTransition in
  directions where no transitions exist.
info: |
  GetNamedTimeZoneNextTransition and GetNamedTimeZonePreviousTransition should
  return null when no transition is found within the search window. This tests
  time zones that have no future transitions and time zones queried from dates
  before any historical transitions.
features: [Temporal]
---*/

// Asia/Riyadh has no DST transitions. It has one historical LMT-to-standard
// transition but no DST. From 2024, there should be no next transition.
var riyadh = Temporal.ZonedDateTime.from("2024-06-15T12:00:00[Asia/Riyadh]");

assert.sameValue(
  riyadh.getTimeZoneTransition("next"),
  null,
  "Asia/Riyadh should have no next time zone transition from 2024"
);

// From a date before any IANA transitions, "previous" should return null.
var earlyRiyadh = Temporal.ZonedDateTime.from("1850-01-01T00:00:00[Asia/Riyadh]");

assert.sameValue(
  earlyRiyadh.getTimeZoneTransition("previous"),
  null,
  "Asia/Riyadh should have no previous transition from 1850"
);
