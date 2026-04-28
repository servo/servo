// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: Using timeStyle, even if dateStyle is present, should throw
features: [Temporal]
---*/

const item = new Temporal.PlainDate(2026, 1, 20);

assert.throws(TypeError, function() {
  item.toLocaleString("en", { dateStyle: "full", timeStyle: "full" });
});
