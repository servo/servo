// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Temporal.Instant string with sub-minute offset
features: [Temporal]
---*/

const epoch = new Temporal.Instant(0n);
const str = "1970-01-01T00:19:32.37+00:19:32.37";

const result1 = Temporal.Instant.compare(str, epoch);
assert.sameValue(result1, 0, "if present, sub-minute offset is accepted exactly (first argument)");

const result2 = Temporal.Instant.compare(epoch, str);
assert.sameValue(result2, 0, "if present, sub-minute offset is accepted exactly (second argument)");
