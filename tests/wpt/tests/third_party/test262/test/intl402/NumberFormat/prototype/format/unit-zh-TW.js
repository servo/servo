// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the unit style.
locale: [zh-TW]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    -987,
    {
      "short": "-987 公里/小時",
      "narrow": "-987公里/小時",
      "long": "每小時 -987 公里",
    }
  ],
  [
    -0.001,
    {
      "short": "-0.001 公里/小時",
      "narrow": "-0.001公里/小時",
      "long": "每小時 -0.001 公里",
    }
  ],
  [
    -0,
    {
      "short": "-0 公里/小時",
      "narrow": "-0公里/小時",
      "long": "每小時 -0 公里",
    }
  ],
  [
    0,
    {
      "short": "0 公里/小時",
      "narrow": "0公里/小時",
      "long": "每小時 0 公里",
    }
  ],
  [
    0.001,
    {
      "short": "0.001 公里/小時",
      "narrow": "0.001公里/小時",
      "long": "每小時 0.001 公里",
    }
  ],
  [
    987,
    {
      "short": "987 公里/小時",
      "narrow": "987公里/小時",
      "long": "每小時 987 公里",
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("zh-TW", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    assert.sameValue(nf.format(number), expected);
  }
}

