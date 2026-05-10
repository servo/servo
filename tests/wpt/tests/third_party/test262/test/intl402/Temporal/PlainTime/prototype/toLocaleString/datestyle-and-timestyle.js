// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Using dateStyle, even if timeStyle is present, should throw
features: [Temporal]
---*/

const item = new Temporal.PlainTime(0, 0);

assert.throws(TypeError, function() {
  item.toLocaleString("en", { dateStyle: "full", timeStyle: "full" });
});
