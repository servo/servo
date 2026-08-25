// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Test TZDB edge case where transitions occur close together
features: [Temporal]
---*/

// Zone America/Noronha (Fernando de Noronha) switched to DST -01:00 according
// to the Brazilian DST rules then in effect on 2000-10-08T00. Then the
// following week on 2000-10-15T00 discarded DST, going back to a permanent
// -02:00 offset. Implementations using a bisect method to find the transition
// times, must use a small enough window to catch these transitions 1 week
// apart.

const noronha = new Temporal.ZonedDateTime(970970400000000000n /* 2000-10-08T01-01:00 */, "America/Noronha");

assert.sameValue(
  noronha.startOfDay().epochNanoseconds,
  noronha.epochNanoseconds,
  "America/Noronha offset transitions close together"
);

// Same deal for zone America/Boa_Vista, but with -03:00 and -04:00
// respectively. Test again with an instance that is not already the start of
// the day.

const boaVistaStart = new Temporal.ZonedDateTime(970977600000000000n /* 2000-10-08T01-03:00 */, "America/Boa_Vista");
const boaVista = new Temporal.ZonedDateTime(970984800000000000n /* 2000-10-08T03-03:00 */, "America/Boa_Vista");

assert.sameValue(
  boaVista.startOfDay().epochNanoseconds,
  boaVistaStart.epochNanoseconds,
  "America/Boa_Vista offset transitions close together"
);

// The same thing occurs in several other Brazilian time zones on the same or
// nearby dates, but these two are sufficient for the test. Other occur at other
// times of the day (see ../getTimeZoneTransition/transitions-close-together.js)
// but those are not relevant for startOfDay().
