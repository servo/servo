// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.month
description: Month property for PlainDateTime
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDateTime(1997, 8, 23, 5, 30, 13)).month, 8);
