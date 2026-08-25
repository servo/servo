// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [ja-JP]
features: [Intl.NumberFormat-unified]
includes: [testIntlNumberFormat.js]
---*/

// The unit/number separator is CLDR data, not specified by ECMA-402, so
// read it from the implementation instead of hardcoding it.
// (e.g. "時速 987 キロメートル" vs. "時速987キロメートル")
const shortSep = getUnitSeparators("ja-JP", "short");
const narrowSep = getUnitSeparators("ja-JP", "narrow");
const longSep = getUnitSeparators("ja-JP", "long");

const tests = [
  [
    -987,
    {
      "short": `-987${shortSep.suffix}km/h`,
      "narrow": `-987${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}-987${longSep.suffix}キロメートル`,
    }
  ],
  [
    -0.001,
    {
      "short": `-0.001${shortSep.suffix}km/h`,
      "narrow": `-0.001${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}-0.001${longSep.suffix}キロメートル`,
    }
  ],
  [
    -0,
    {
      "short": `-0${shortSep.suffix}km/h`,
      "narrow": `-0${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}-0${longSep.suffix}キロメートル`,
    }
  ],
  [
    0,
    {
      "short": `0${shortSep.suffix}km/h`,
      "narrow": `0${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}0${longSep.suffix}キロメートル`,
    }
  ],
  [
    0.001,
    {
      "short": `0.001${shortSep.suffix}km/h`,
      "narrow": `0.001${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}0.001${longSep.suffix}キロメートル`,
    }
  ],
  [
    987,
    {
      "short": `987${shortSep.suffix}km/h`,
      "narrow": `987${narrowSep.suffix}km/h`,
      "long": `時速${longSep.prefix}987${longSep.suffix}キロメートル`,
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("ja-JP", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}
