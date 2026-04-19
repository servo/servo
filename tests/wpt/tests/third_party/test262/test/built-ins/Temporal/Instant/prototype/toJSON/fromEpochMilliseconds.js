// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tojson
description: Various test cases using fromEpochMilliseconds.
features: [Temporal]
---*/

assert.sameValue(Temporal.Instant.fromEpochMilliseconds(0).toJSON(), '1970-01-01T00:00:00Z');
let days_in_ms = 24 * 60 * 60 * 1000;
assert.sameValue(Temporal.Instant.fromEpochMilliseconds(365 * days_in_ms - 1).toJSON(), '1970-12-31T23:59:59.999Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds(365 * days_in_ms).toJSON(), '1971-01-01T00:00:00Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds(2 * 365 * days_in_ms - 1).toJSON(), '1971-12-31T23:59:59.999Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds(2 * 365 * days_in_ms).toJSON(), '1972-01-01T00:00:00Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds((2 * 365 + 58) * days_in_ms).toJSON(), '1972-02-28T00:00:00Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds((2 * 365 + 59) * days_in_ms).toJSON(), '1972-02-29T00:00:00Z');
assert.sameValue(Temporal.Instant.fromEpochMilliseconds((15 * 365 + 4) * days_in_ms).toJSON(), '1985-01-01T00:00:00Z');
