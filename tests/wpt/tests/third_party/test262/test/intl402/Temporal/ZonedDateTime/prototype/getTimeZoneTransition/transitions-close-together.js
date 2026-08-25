// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Test TZDB edge case where transitions occur close together
features: [Temporal]
---*/

// This is a list of time zones that have transitions occurring less than two
// weeks apart. (Some additional ones occur in questionable future calculations
// which are not included here.)
// Implementations using a bisect strategy to find the transition times, must
// use a small enough window to catch these transitions.

const testData = {
  "Africa/Tunis": [-842918400n /* 1943-04-17T01+01:00 */, -842223600n /* 1943-04-25T03+02:00 */],
  "America/Argentina/Tucuman": [1086058800n /* 2004-05-31T23-04:00 */, 1087099200n /* 2004-06-13T01-03:00 */],
  "America/Boa_Vista": [970977600n /* 2000-10-08T01-03:00 */, 971578800n /* 2000-10-14T23-04:00 */],
  "America/Fortaleza": [970974000n /* 2000-10-08T01-02:00 */, 972180000n /* 2000-10-21T23-03:00 */],
  "America/Maceio": [970974000n /* 2000-10-08T01-02:00 */, 972180000n /* 2000-10-21T23-03:00 */],
  "America/Noronha": [970970400n /* 2000-10-08T01-01:00 */, 971571600n /* 2000-10-14T23-02:00 */],
  "America/Recife": [970974000n /* 2000-10-08T01-02:00 */, 971575200n /* 2000-10-14T23-03:00 */],
  "Europe/Riga": [-796777200n /* 1944-10-02T02+01:00 */, -795834000n /* 1944-10-13T02+03:00 */],
  "Europe/Simferopol": [-812502000n /* 1944-04-03T03+02:00 */, -811648800n /* 1944-04-13T01+03:00 */],
  "Europe/Tirane": [-844556400n /* 1943-03-29T03+02:00 */, -843519600n /* 1943-04-10T02+01:00 */],
  "Europe/Vienna": [-781052400n /* 1945-04-02T03+02:00 */, -780188400n /* 1945-04-12T02+01:00 */],
}

for (const [zone, [first, second]] of Object.entries(testData)) {
  for (const [label, epochSeconds] of Object.entries({ first, second }) ) {
    const transition = new Temporal.ZonedDateTime(epochSeconds * 1_000_000_000n, zone);
    const before = new Temporal.ZonedDateTime((epochSeconds - 1800n) * 1_000_000_000n, zone);
    const after = new Temporal.ZonedDateTime((epochSeconds + 1800n) * 1_000_000_000n, zone);

    assert.sameValue(
      before.getTimeZoneTransition("next").epochNanoseconds,
      transition.epochNanoseconds,
      `${zone} offset transitions close together, next to ${label} transition`
    );
    assert.sameValue(
      after.getTimeZoneTransition("previous").epochNanoseconds,
      transition.epochNanoseconds,
      `${zone} offset transitions close together, previous to ${label} transition`
    );
  }
}
