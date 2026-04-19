// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: Appropriate error thrown when object argument is invalid
features: [Temporal]
---*/

const instance = Temporal.PlainDate.from({ year: 2000, month: 5, day: 2 });

// End up in ISODateFromFields with missing fields
assert.throws(TypeError, () => instance.equals({}), "plain object");
assert.throws(TypeError, () => instance.equals({ year: 1972, month: 7 }), "only year, month");
assert.throws(TypeError, () => instance.equals({ year: 1972, month: 7 }), "only year, month");
assert.throws(TypeError, () => instance.equals({ year: 1972, day: 7 }), "only year, day");
assert.throws(TypeError, () => instance.equals(Temporal.PlainDate), "Temporal.PlainDate");

// Tries to get fields with an invalid receiver
assert.throws(TypeError, () => instance.equals(Temporal.PlainDate.prototype), "Temporal.PlainDate.prototype");
