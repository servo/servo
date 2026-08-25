// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.epochnanoseconds
description: The "epochNanoseconds" property of Temporal.ZonedDateTime.prototype
features: [Temporal]
---*/

const descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "epochNanoseconds");
assert.sameValue(typeof descriptor.get, "function");
assert.sameValue(descriptor.set, undefined);
assert.sameValue(descriptor.enumerable, false);
assert.sameValue(descriptor.configurable, true);
