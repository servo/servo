// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const duration1 = new Temporal.Duration(1);
const duration2 = new Temporal.Duration(0, 12);
assert.throws(RangeError, () => Temporal.Duration.compare(duration1, duration2), "default relativeTo is undefined");
assert.throws(RangeError, () => Temporal.Duration.compare(duration1, duration2, undefined), "default relativeTo is undefined");
