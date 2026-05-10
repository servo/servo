// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Checking disambiguation options for daylight savings time changes
features: [Temporal]
---*/

const tz = "America/Vancouver";

const dt1 = new Temporal.PlainDateTime(2000, 4, 2, 2);

const zdt1 = dt1.toZonedDateTime(tz);
const zdt1_compatible = dt1.toZonedDateTime(tz, { disambiguation: "compatible" });
const zdt1_earlier = dt1.toZonedDateTime(tz, { disambiguation: "earlier" });
const zdt1_later = dt1.toZonedDateTime(tz, { disambiguation: "later" });

assert.sameValue(zdt1.epochNanoseconds, 954669600000000000n, "Fall DST (no disambiguation)");
assert.sameValue(zdt1_compatible.epochNanoseconds, 954669600000000000n, "Fall DST (disambiguation = compatible)");
assert.sameValue(zdt1_earlier.epochNanoseconds, 954666000000000000n, "Fall DST (disambiguation = earlier)");
assert.sameValue(zdt1_later.epochNanoseconds, 954669600000000000n, "Fall DST (disambiguation = later)");

assert.throws(
  RangeError,
  () => dt1.toZonedDateTime(tz, { disambiguation: "reject" }),
  "Fall DST (disambiguation = reject)"
);

const dt2 = new Temporal.PlainDateTime(2000, 10, 29, 1);

const zdt2 = dt2.toZonedDateTime(tz);
const zdt2_compatible = dt2.toZonedDateTime(tz, { disambiguation: "compatible" });
const zdt2_earlier = dt2.toZonedDateTime(tz, { disambiguation: "earlier" });
const zdt2_later = dt2.toZonedDateTime(tz, { disambiguation: "later" });

assert.sameValue(zdt2.epochNanoseconds, 972806400000000000n, "Spring DST (no disambiguation)");
assert.sameValue(zdt2_compatible.epochNanoseconds, 972806400000000000n, "Spring DST (disambiguation = compatible)");
assert.sameValue(zdt2_earlier.epochNanoseconds, 972806400000000000n, "Spring DST (disambiguation = earlier)");
assert.sameValue(zdt2_later.epochNanoseconds, 972810000000000000n, "Spring DST (disambiguation = later)");

assert.throws(
  RangeError,
  () => dt2.toZonedDateTime(tz, { disambiguation: "reject" }),
  "Spring DST (disambiguation = reject)"
);
