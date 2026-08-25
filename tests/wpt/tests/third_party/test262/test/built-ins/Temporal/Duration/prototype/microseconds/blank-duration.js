// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.microseconds
description: Behaviour with blank duration
features: [Temporal]
---*/

const blank = new Temporal.Duration();
assert.sameValue(blank.microseconds, 0);
