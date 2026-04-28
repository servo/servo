// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Throw if sole argument is undefined
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);

assert.throws(
  TypeError,
  () => dt.round(undefined),
  "throws without undefined as sole parameter"
);
