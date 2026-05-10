// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaindate
description: basic tests for toPlainDate
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let d1 = new Temporal.PlainDateTime(2021, 12, 11, 1, 2, 3, 4, 5, 6);
TemporalHelpers.assertPlainDate(d1.toPlainDate(), 2021, 12, 'M12', 11);
