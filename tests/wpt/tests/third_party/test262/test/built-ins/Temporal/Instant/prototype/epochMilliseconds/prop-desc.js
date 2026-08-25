// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.instant.prototype.epochmilliseconds
description: The "epochMilliseconds" property of Temporal.Instant.prototype
features: [Temporal]
---*/

const descriptor = Object.getOwnPropertyDescriptor(Temporal.Instant.prototype, "epochMilliseconds");
assert.sameValue(typeof descriptor.get, "function");
assert.sameValue(descriptor.set, undefined);
assert.sameValue(descriptor.enumerable, false);
assert.sameValue(descriptor.configurable, true);
