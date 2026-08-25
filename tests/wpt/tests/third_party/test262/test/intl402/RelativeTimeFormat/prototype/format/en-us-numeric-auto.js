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

const rtf = new Intl.RelativeTimeFormat("en-US", { "numeric": "auto" });

assert.sameValue(typeof rtf.format, "function", "format should be supported");

// https://www.unicode.org/cldr/charts/33/summary/en.html#1530
const exceptions = {
  "year": {
    "-1": "last year",
    "0": "this year",
    "1": "next year",
  },
  "quarter": {
    "-1": "last quarter",
    "0": "this quarter",
    "1": "next quarter",
  },
  "month": {
    "-1": "last month",
    "0": "this month",
    "1": "next month",
  },
  "week": {
    "-1": "last week",
    "0": "this week",
    "1": "next week",
  },
  "day": {
    "-1": "yesterday",
    "0": "today",
    "1": "tomorrow",
  },
  "hour": {
    "-1": "1 hour ago",
    '0': 'this hour',
    "1": "in 1 hour",
  },
  "minute": {
    "-1": "1 minute ago",
    '0': 'this minute',
    "1": "in 1 minute",
  },
  "second": {
    "-1": "1 second ago",
    "0": "now",
    "1": "in 1 second",
  },
};

for (const unit of units) {
  const expected = unit in exceptions
    ? [exceptions[unit]["1"], exceptions[unit]["0"], exceptions[unit]["0"], exceptions[unit]["-1"]]
    : [`in 1 ${unit}`, `in 0 ${unit}s`, `0 ${unit}s ago`, `1 ${unit} ago`];

  assert.sameValue(rtf.format(1000, unit), `in 1,000 ${unit}s`);
  assert.sameValue(rtf.format(10, unit), `in 10 ${unit}s`);
  assert.sameValue(rtf.format(2, unit), `in 2 ${unit}s`);
  assert.sameValue(rtf.format(1, unit), expected[0]);
  assert.sameValue(rtf.format(0, unit), expected[1]);
  assert.sameValue(rtf.format(-0, unit), expected[2]);
  assert.sameValue(rtf.format(-1, unit), expected[3]);
  assert.sameValue(rtf.format(-2, unit), `2 ${unit}s ago`);
  assert.sameValue(rtf.format(-10, unit), `10 ${unit}s ago`);
  assert.sameValue(rtf.format(-1000, unit), `1,000 ${unit}s ago`);
}
