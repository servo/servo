// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [zh-TW]
features: [Intl.NumberFormat-unified]
includes: [testIntlNumberFormat.js]
---*/

// The unit/number separator is CLDR data, not specified by ECMA-402, so
// read it from the implementation instead of hardcoding it.
// (e.g. "-987 公里/小時" vs. "-987公里/小時")
const shortSep = getUnitSeparators("zh-TW", "short");
const narrowSep = getUnitSeparators("zh-TW", "narrow");
const longSep = getUnitSeparators("zh-TW", "long");

const tests = [
  [
    -987,
    {
      "short": `-987${shortSep.suffix}公里/小時`,
      "narrow": `-987${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}-987${longSep.suffix}公里`,
    }
  ],
  [
    -0.001,
    {
      "short": `-0.001${shortSep.suffix}公里/小時`,
      "narrow": `-0.001${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}-0.001${longSep.suffix}公里`,
    }
  ],
  [
    -0,
    {
      "short": `-0${shortSep.suffix}公里/小時`,
      "narrow": `-0${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}-0${longSep.suffix}公里`,
    }
  ],
  [
    0,
    {
      "short": `0${shortSep.suffix}公里/小時`,
      "narrow": `0${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}0${longSep.suffix}公里`,
    }
  ],
  [
    0.001,
    {
      "short": `0.001${shortSep.suffix}公里/小時`,
      "narrow": `0.001${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}0.001${longSep.suffix}公里`,
    }
  ],
  [
    987,
    {
      "short": `987${shortSep.suffix}公里/小時`,
      "narrow": `987${narrowSep.suffix}公里/小時`,
      "long": `每小時${longSep.prefix}987${longSep.suffix}公里`,
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("zh-TW", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}
