// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Temporal.ZonedDateTime.from({}) throws.
features: [Temporal]
---*/

// Temporal.ZonedDateTime.from({}) throws
assert.throws(TypeError, () => Temporal.ZonedDateTime.from({}))
