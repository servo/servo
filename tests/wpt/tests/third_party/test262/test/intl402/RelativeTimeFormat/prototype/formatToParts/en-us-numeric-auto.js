// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Checks the behavior of Intl.RelativeTimeFormat.prototype.formatToParts() in English.
features: [Intl.RelativeTimeFormat]
locale: [en-US]
---*/

function verifyFormatParts(actual, expected, message) {
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (let i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type, `${message}: parts[${i}].type`);
    assert.sameValue(actual[i].value, expected[i].value, `${message}: parts[${i}].value`);
    assert.sameValue(actual[i].unit, expected[i].unit, `${message}: parts[${i}].unit`);
  }
}

function expected(key, unit, default_) {
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
      "0": "this hour",
    },
    "minute": {
      "0": "this minute",
    },
    "second": {
      "0": "now",
    },
  };

  const exception = exceptions[unit] || {};
  if (key in exception) {
    return [
      { "type": "literal", "value": exception[key] },
    ]
  }

  return default_;
}

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

assert.sameValue(typeof rtf.formatToParts, "function", "formatToParts should be supported");

for (const unit of units) {
  verifyFormatParts(rtf.formatToParts(1000, unit), [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "1", "unit": unit },
    { "type": "group", "value": ",", "unit": unit },
    { "type": "integer", "value": "000", "unit": unit },
    { "type": "literal", "value": ` ${unit}s` },
  ], `formatToParts(1000, ${unit})`);

  verifyFormatParts(rtf.formatToParts(10, unit), [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "10", "unit": unit },
    { "type": "literal", "value": ` ${unit}s` },
  ], `formatToParts(10, ${unit})`);

  verifyFormatParts(rtf.formatToParts(2, unit), [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "2", "unit": unit },
    { "type": "literal", "value": ` ${unit}s` },
  ], `formatToParts(2, ${unit})`);

  verifyFormatParts(rtf.formatToParts(1, unit), expected("1", unit, [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "1", "unit": unit },
    { "type": "literal", "value": ` ${unit}` },
  ]), `formatToParts(1, ${unit})`);

  verifyFormatParts(rtf.formatToParts(0, unit), expected("0", unit, [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "0", "unit": unit },
    { "type": "literal", "value": ` ${unit}s` },
  ]), `formatToParts(0, ${unit})`);

  verifyFormatParts(rtf.formatToParts(-0, unit), expected("0", unit, [
    { "type": "integer", "value": "0", "unit": unit },
    { "type": "literal", "value": ` ${unit}s ago` },
  ]), `formatToParts(-0, ${unit})`);

  verifyFormatParts(rtf.formatToParts(-1, unit), expected("-1", unit, [
    { "type": "integer", "value": "1", "unit": unit },
    { "type": "literal", "value": ` ${unit} ago` },
  ]), `formatToParts(-1, ${unit})`);

  verifyFormatParts(rtf.formatToParts(-2, unit), [
    { "type": "integer", "value": "2", "unit": unit },
    { "type": "literal", "value": ` ${unit}s ago` },
  ], `formatToParts(-2, ${unit})`);

  verifyFormatParts(rtf.formatToParts(-10, unit), [
    { "type": "integer", "value": "10", "unit": unit },
    { "type": "literal", "value": ` ${unit}s ago` },
  ], `formatToParts(-10, ${unit})`);

  verifyFormatParts(rtf.formatToParts(-1000, unit), [
    { "type": "integer", "value": "1", "unit": unit },
    { "type": "group", "value": ",", "unit": unit },
    { "type": "integer", "value": "000", "unit": unit },
    { "type": "literal", "value": ` ${unit}s ago` },
  ], `formatToParts(-1000, ${unit})`);

  verifyFormatParts(rtf.formatToParts(123456.78, unit), [
    { "type": "literal", "value": "in " },
    { "type": "integer", "value": "123", "unit": unit },
    { "type": "group", "value": ",", "unit": unit },
    { "type": "integer", "value": "456", "unit": unit },
    { "type": "decimal", "value": ".", "unit": unit },
    { "type": "fraction", "value": "78", "unit": unit },
    { "type": "literal", "value": ` ${unit}s` },
  ], `formatToParts(123456.78, ${unit})`);
}
