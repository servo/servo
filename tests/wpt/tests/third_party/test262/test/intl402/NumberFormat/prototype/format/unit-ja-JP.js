// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [ja-JP]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    -987,
    {
      "short": "-987 km/h",
      "narrow": "-987km/h",
      "long": "時速 -987 キロメートル",
    }
  ],
  [
    -0.001,
    {
      "short": "-0.001 km/h",
      "narrow": "-0.001km/h",
      "long": "時速 -0.001 キロメートル",
    }
  ],
  [
    -0,
    {
      "short": "-0 km/h",
      "narrow": "-0km/h",
      "long": "時速 -0 キロメートル",
    }
  ],
  [
    0,
    {
      "short": "0 km/h",
      "narrow": "0km/h",
      "long": "時速 0 キロメートル",
    }
  ],
  [
    0.001,
    {
      "short": "0.001 km/h",
      "narrow": "0.001km/h",
      "long": "時速 0.001 キロメートル",
    }
  ],
  [
    987,
    {
      "short": "987 km/h",
      "narrow": "987km/h",
      "long": "時速 987 キロメートル",
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("ja-JP", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}

