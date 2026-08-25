// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the behavior of Intl.RelativeTimeFormat.prototype.format() in English.
features: [Intl.RelativeTimeFormat]
locale: [en-US]
---*/

const units = [
  "second",
  "minute",
  "hour",
  "day",
  "week",
  "month",
  "quarter",
  "year",
];

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.format, "function", "format should be supported");

for (const unit of units) {
  assert.sameValue(rtf.format(1000, unit), `in 1,000 ${unit}s`);
  assert.sameValue(rtf.format(10, unit), `in 10 ${unit}s`);
  assert.sameValue(rtf.format(2, unit), `in 2 ${unit}s`);
  assert.sameValue(rtf.format(1, unit), `in 1 ${unit}`);
  assert.sameValue(rtf.format(0, unit), `in 0 ${unit}s`);
  assert.sameValue(rtf.format(-0, unit), `0 ${unit}s ago`);
  assert.sameValue(rtf.format(-1, unit), `1 ${unit} ago`);
  assert.sameValue(rtf.format(-2, unit), `2 ${unit}s ago`);
  assert.sameValue(rtf.format(-10, unit), `10 ${unit}s ago`);
  assert.sameValue(rtf.format(-1000, unit), `1,000 ${unit}s ago`);
}
