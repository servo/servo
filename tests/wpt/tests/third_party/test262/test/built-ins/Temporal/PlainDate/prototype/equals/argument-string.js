// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: equals with a valid string
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);
assert.sameValue(instance.equals("2000-05-02"), true, "same date");
assert.sameValue(instance.equals("2000-05-04"), false, "different date");
