// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.monthcode
description: monthCode property for PlainDate
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDate(2021, 7, 15)).monthCode, 'M07');
assert.sameValue(Temporal.PlainDate.from('2019-03-15').monthCode, 'M03');
