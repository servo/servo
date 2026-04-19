// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the behavior of Intl.RelativeTimeFormat.prototype.format() in English.
features: [Intl.RelativeTimeFormat]
locale: [en-US]
---*/

const units = {
  "second": ["sec."],
  "minute": ["min."],
  "hour": ["hr."],
  "day": ["day", "days"],
  "week": ["wk."],
  "month": ["mo."],
  "quarter": ["qtr.", "qtrs."],
  "year": ["yr."],
};

const rtf = new Intl.RelativeTimeFormat("en-US", {
  "style": "short",
});

assert.sameValue(typeof rtf.format, "function", "format should be supported");

for (const [unitArgument, unitStrings] of Object.entries(units)) {
  const [singular, plural = singular] = unitStrings;
  assert.sameValue(rtf.format(1000, unitArgument), `in 1,000 ${plural}`);
  assert.sameValue(rtf.format(10, unitArgument), `in 10 ${plural}`);
  assert.sameValue(rtf.format(2, unitArgument), `in 2 ${plural}`);
  assert.sameValue(rtf.format(1, unitArgument), `in 1 ${singular}`);
  assert.sameValue(rtf.format(0, unitArgument), `in 0 ${plural}`);
  assert.sameValue(rtf.format(-0, unitArgument), `0 ${plural} ago`);
  assert.sameValue(rtf.format(-1, unitArgument), `1 ${singular} ago`);
  assert.sameValue(rtf.format(-2, unitArgument), `2 ${plural} ago`);
  assert.sameValue(rtf.format(-10, unitArgument), `10 ${plural} ago`);
  assert.sameValue(rtf.format(-1000, unitArgument), `1,000 ${plural} ago`);
}
