// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Basic tests for disambiguation option, with DST time zone
features: [Temporal]
---*/

const dtmFall = new Temporal.PlainDateTime(2000, 10, 29, 1, 45);

assert.sameValue(
  dtmFall.toZonedDateTime("America/Los_Angeles").epochNanoseconds,
  972809100_000_000_000n,  // 2000-10-29T08:45:00Z
  "epoch nanoseconds in fall back - no disambiguation"
);

assert.sameValue(
  dtmFall.toZonedDateTime("America/Los_Angeles", { disambiguation: "earlier" }).epochNanoseconds,
  972809100_000_000_000n,  // 2000-10-29T08:45:00Z
  "epoch nanoseconds in fall back - earlier"
);

assert.sameValue(
  dtmFall.toZonedDateTime("America/Los_Angeles", { disambiguation: "later" }).epochNanoseconds,
  972812700_000_000_000n,  // 2000-10-29T09:45:00Z
  "epoch nanoseconds in fall back - later"
);

assert.sameValue(
  dtmFall.toZonedDateTime("America/Los_Angeles", { disambiguation: "compatible" }).epochNanoseconds,
  972809100_000_000_000n,  // 2000-10-29T08:45:00Z
  "epoch nanoseconds in fall back - compatible"
);

assert.throws(
  RangeError,
  () => dtmFall.toZonedDateTime("America/Los_Angeles", { disambiguation: "reject" }),
  "fall back - reject"
);

var dtmSpring = new Temporal.PlainDateTime(2000, 4, 2, 2, 30);

assert.sameValue(
  dtmSpring.toZonedDateTime("America/Los_Angeles").epochNanoseconds,
  954671400_000_000_000n,  // 2000-04-02T10:30:00Z
  "epoch nanoseconds in spring forward - no disambiguation"
);

assert.sameValue(
  dtmSpring.toZonedDateTime("America/Los_Angeles", { disambiguation: "earlier" }).epochNanoseconds,
  954667800_000_000_000n,  // 2000-04-02T09:30:00Z
  "epoch nanoseconds in spring forward - earlier"
);

assert.sameValue(
  dtmSpring.toZonedDateTime("America/Los_Angeles", { disambiguation: "later" }).epochNanoseconds,
  954671400_000_000_000n,  // 2000-04-02T10:30:00Z
  "epoch nanoseconds in spring forward - later"
);

assert.sameValue(
  dtmSpring.toZonedDateTime("America/Los_Angeles", { disambiguation: "compatible" }).epochNanoseconds,
  954671400_000_000_000n,  // 2000-04-02T10:30:00Z
  "epoch nanoseconds in spring forward - compatible"
);

assert.throws(
  RangeError,
  () => dtmSpring.toZonedDateTime("America/Los_Angeles", { disambiguation: "reject" }),
  "spring forward - reject"
);
