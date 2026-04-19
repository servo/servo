// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.prototype.since
description: Reversibility of differences.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

/*
const earlier = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789-03:00[-03:00]");
const later = Temporal.ZonedDateTime.from("2019-10-29T10:46:38.271986102-03:00[-03:00]");
*/
const earlier = new Temporal.ZonedDateTime(217189410123456789n, "-03:00");
const later = new Temporal.ZonedDateTime(1572356798271986102n, "-03:00");

[
  "hours",
  "minutes",
  "seconds"
].forEach(largestUnit => {
    const diff = later.since(earlier, { largestUnit });
    TemporalHelpers.assertDurationsEqual(earlier.since(later, { largestUnit }),
                                         diff.negated())
    TemporalHelpers.assertDurationsEqual(earlier.until(later, { largestUnit }),
                                         diff);
    // difference symmetrical with regard to negative durations
    assert(earlier.subtract(diff.negated()).equals(later));
    assert(later.add(diff.negated()).equals(earlier));
});

[
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds"
].forEach(largestUnit => {
  const diff1 = earlier.until(later, { largestUnit });
  const diff2 = later.since(earlier, { largestUnit });
  assert(earlier.add(diff1).equals(later));
  assert(later.subtract(diff2).equals(earlier));
});
