// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the unit style.
locale: [zh-TW]
features: [Intl.NumberFormat-unified]
includes: [testIntlNumberFormat.js]
---*/

function verifyFormatParts(actual, expected, message) {
  assert.sameValue(Array.isArray(expected), true, `${message}: expected is Array`);
  assert.sameValue(Array.isArray(actual), true, `${message}: actual is Array`);
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (let i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type, `${message}: parts[${i}].type`);
    assert.sameValue(actual[i].value, expected[i].value, `${message}: parts[${i}].value`);
  }
}

function makePart(type, value) {
  return value ? [{ "type": type, "value": value }] : [];
}

function addUnitParts(numberParts, separators, prefixText, suffixText) {
  return [].concat(
    makePart("unit", prefixText),
    makePart("literal", separators.prefix),
    numberParts,
    makePart("literal", separators.suffix),
    makePart("unit", suffixText)
  );
}

// The unit/number separator is CLDR data, not specified by ECMA-402, so
// read it from the implementation instead of hardcoding it.
// (e.g. a literal " " part between the number and unit parts, or none)
const shortSep = getUnitSeparators("zh-TW", "short");
const narrowSep = getUnitSeparators("zh-TW", "narrow");
const longSep = getUnitSeparators("zh-TW", "long");

const tests = [
  [
    -987,
    [{ type: "minusSign", value: "-" }, { type: "integer", value: "987" }],
  ],
  [
    -0.001,
    [{ type: "minusSign", value: "-" }, { type: "integer", value: "0" }, { type: "decimal", value: "." }, { type: "fraction", value: "001" }],
  ],
  [
    -0,
    [{ type: "minusSign", value: "-" }, { type: "integer", value: "0" }],
  ],
  [
    0,
    [{ type: "integer", value: "0" }],
  ],
  [
    0.001,
    [{ type: "integer", value: "0" }, { type: "decimal", value: "." }, { type: "fraction", value: "001" }],
  ],
  [
    987,
    [{ type: "integer", value: "987" }],
  ],
];

for (const [number, numberParts] of tests) {
  const expectedData = {
    "short": addUnitParts(numberParts, shortSep, "", "公里/小時"),
    "narrow": addUnitParts(numberParts, narrowSep, "", "公里/小時"),
    "long": addUnitParts(numberParts, longSep, "每小時", "公里"),
  };

  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("zh-TW", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    verifyFormatParts(nf.formatToParts(number), expected, `${number} ${unitDisplay}`);
  }
}
