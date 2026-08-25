// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Calls to PYM.until throw when missing required arguments.
features: [Temporal]
---*/

const jun13 = new Temporal.PlainYearMonth(2013, 6);

assert.throws(TypeError, () => jun13.until({ year: 1994 }), 'Throws when missing required month');
assert.throws(TypeError, () => jun13.until({ month: 11 }), 'Throws when missing required year');
