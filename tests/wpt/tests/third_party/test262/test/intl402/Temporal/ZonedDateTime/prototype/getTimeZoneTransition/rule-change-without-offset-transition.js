// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Rule changes in the TZDB that do not have offset transtions should not be
  returned by getTimeZoneTransition.
features: [Temporal]
---*/

// Europe/London changed from DST to permanent British Standard Time on
// 1968-10-27, but the actual UTC offset did not change at that time.
// getTimeZoneTransition should not return an instant on 1968-10-27.

const londonPrev = new Temporal.ZonedDateTime(0n, "Europe/London")
  .getTimeZoneTransition("previous");
assert.notSameValue(
  londonPrev.offsetNanoseconds,
  londonPrev.subtract({ nanoseconds: 1 }).offsetNanoseconds,
  "should be a UTC offset transition"
);
assert.sameValue(
  londonPrev.epochNanoseconds,
  -59004000000000000n,
  "epoch nanoseconds for 1968-02-18T03:00:00+01:00"
);

const londonNext = new Temporal.ZonedDateTime(-39488400000000000n /* 1968-10-01T00:00:00+01:00 */, "Europe/London")
  .getTimeZoneTransition("next");
assert.notSameValue(
  londonNext.offsetNanoseconds,
  londonNext.subtract({ nanoseconds: 1 }).offsetNanoseconds,
  "should be a UTC offset transition"
);
assert.sameValue(
  londonNext.epochNanoseconds,
  57722400000000000n,
  "epoch nanoseconds for 1971-10-31T02:00:00+00:00"
);

// Similarly, America/Anchorage changed from DST to permanent standard time on
// 1967-04-01. The UTC offset did not change, but the abbreviation did (AST to
// AHST). Still, getTimeZoneTransition should not return an instant on 1967-04-01

const anchoragePrev = new Temporal.ZonedDateTime(-84290400000000000n /* 1967-05-01T00:00:00-10:00 */, "America/Anchorage")
  .getTimeZoneTransition("previous");
assert.notSameValue(
  anchoragePrev.offsetNanoseconds,
  anchoragePrev.subtract({ nanoseconds: 1 }).offsetNanoseconds,
  "should be a UTC offset transition"
);
assert.sameValue(
  anchoragePrev.epochNanoseconds,
  -765378000000000000n,
  "epoch nanoseconds for 1945-09-30T01:00:00-10:00"
);

const anchorageNext = new Temporal.ZonedDateTime(-94658400000000000n /* 1967-01-01T00:00:00-10:00 */, "America/Anchorage")
  .getTimeZoneTransition("next");
assert.notSameValue(
  anchorageNext.offsetNanoseconds,
  anchorageNext.subtract({ nanoseconds: 1 }).offsetNanoseconds,
  "should be a UTC offset transition"
);
assert.sameValue(
  anchorageNext.epochNanoseconds,
  -21470400000000000n,
  "epoch nanoseconds for 1969-04-27T03:00:00-09:00"
);
