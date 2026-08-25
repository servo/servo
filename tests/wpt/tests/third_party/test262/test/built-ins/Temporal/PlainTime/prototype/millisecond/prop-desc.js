// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaintime.prototype.millisecond
description: The "millisecond" property of Temporal.PlainTime.prototype
features: [Temporal]
---*/

const descriptor = Object.getOwnPropertyDescriptor(Temporal.PlainTime.prototype, "millisecond");
assert.sameValue(typeof descriptor.get, "function");
assert.sameValue(descriptor.set, undefined);
assert.sameValue(descriptor.enumerable, false);
assert.sameValue(descriptor.configurable, true);
