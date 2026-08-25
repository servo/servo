// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: since() works.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1976-11-18T15:23:30.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");
const diff = later.since(earlier);

TemporalHelpers.assertDurationsEqual(earlier.since(later), diff.negated());
TemporalHelpers.assertDurationsEqual(earlier.until(later), diff);
TemporalHelpers.assertInstantsEqual(earlier.add(diff), later);
TemporalHelpers.assertInstantsEqual(later.subtract(diff), earlier);
