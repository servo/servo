// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: Temporal.ZonedDateTime constructor cannot be called as a function
info: |
    1. If NewTarget is undefined, throw a TypeError exception.
features: [Temporal]
---*/

assert.throws(TypeError, () => Temporal.ZonedDateTime(0n, "UTC"));
