// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [de-DE]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    -987,
    {
      "short": "-987 km/h",
      "narrow": "-987 km/h",
      "long": "-987 Kilometer pro Stunde",
    }
  ],
  [
    -0.001,
    {
      "short": "-0,001 km/h",
      "narrow": "-0,001 km/h",
      "long": "-0,001 Kilometer pro Stunde",
    }
  ],
  [
    -0,
    {
      "short": "-0 km/h",
      "narrow": "-0 km/h",
      "long": "-0 Kilometer pro Stunde",
    }
  ],
  [
    0,
    {
      "short": "0 km/h",
      "narrow": "0 km/h",
      "long": "0 Kilometer pro Stunde",
    }
  ],
  [
    0.001,
    {
      "short": "0,001 km/h",
      "narrow": "0,001 km/h",
      "long": "0,001 Kilometer pro Stunde",
    }
  ],
  [
    987,
    {
      "short": "987 km/h",
      "narrow": "987 km/h",
      "long": "987 Kilometer pro Stunde",
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("de-DE", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}

