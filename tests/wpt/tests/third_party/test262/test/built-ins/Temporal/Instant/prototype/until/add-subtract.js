// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: until() works.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1969-07-24T16:50:35.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");
const diff = earlier.until(later);

TemporalHelpers.assertDurationsEqual(later.until(earlier), diff.negated());
TemporalHelpers.assertDurationsEqual(later.since(earlier), diff);
TemporalHelpers.assertInstantsEqual(earlier.add(diff), later);
TemporalHelpers.assertInstantsEqual(later.subtract(diff), earlier);

