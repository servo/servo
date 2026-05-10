// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Instant strings with UTC offset fractional part are not confused with time fractional part
features: [Temporal]
---*/

const epoch = new Temporal.Instant(0n);
const str = "1970-01-01T00:02:00.000000000+00:02[+01:30]";

assert.sameValue(Temporal.Instant.compare(str, epoch), 0, "UTC offset determined from offset part of string (first argument)");
assert.sameValue(Temporal.Instant.compare(epoch, str), 0, "UTC offset determined from offset part of string (second argument)");
