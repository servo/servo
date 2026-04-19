// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [ko-KR]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    -987,
    {
      "short": "-987km/h",
      "narrow": "-987km/h",
      "long": "시속 -987킬로미터",
    }
  ],
  [
    -0.001,
    {
      "short": "-0.001km/h",
      "narrow": "-0.001km/h",
      "long": "시속 -0.001킬로미터",
    }
  ],
  [
    -0,
    {
      "short": "-0km/h",
      "narrow": "-0km/h",
      "long": "시속 -0킬로미터",
    }
  ],
  [
    0,
    {
      "short": "0km/h",
      "narrow": "0km/h",
      "long": "시속 0킬로미터",
    }
  ],
  [
    0.001,
    {
      "short": "0.001km/h",
      "narrow": "0.001km/h",
      "long": "시속 0.001킬로미터",
    }
  ],
  [
    987,
    {
      "short": "987km/h",
      "narrow": "987km/h",
      "long": "시속 987킬로미터",
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("ko-KR", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}

