// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the percent style and unit.
locale: [en-US]
features: [Intl.NumberFormat-unified]
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

const nfStyle = new Intl.NumberFormat("en-US", { style: "percent" });
verifyFormatParts(nfStyle.formatToParts(-123), [
  {"type":"minusSign","value":"-"},
  {"type":"integer","value":"12"},
  {"type":"group","value":","},
  {"type":"integer","value":"300"},
  {"type":"percentSign","value":"%"},
], "style");

const nfUnit = new Intl.NumberFormat("en-US", { style: "unit", unit: "percent" });
verifyFormatParts(nfUnit.formatToParts(-123), [
  {"type":"minusSign","value":"-"},
  {"type":"integer","value":"123"},
  {"type":"unit","value":"%"},
], "unit");
