// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: TypeError thrown if an invalid property bag passed
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(2000, 5, 2);
assert.throws(TypeError, () => plainDate.toPlainDateTime({}), "empty object");
assert.throws(TypeError, () => plainDate.toPlainDateTime({ minutes: 30 }), "plural property");
