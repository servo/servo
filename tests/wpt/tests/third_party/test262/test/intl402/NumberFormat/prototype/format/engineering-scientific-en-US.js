// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the engineering and scientific notations.
locale: [en-US]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    0.000345,
    "345E-6",
    "3.45E-4",
  ],
  [
    0.345,
    "345E-3",
    "3.45E-1",
  ],
  [
    3.45,
    "3.45E0",
    "3.45E0",
  ],
  [
    34.5,
    "34.5E0",
    "3.45E1",
  ],
  [
    543,
    "543E0",
    "5.43E2",
  ],
  [
    5430,
    "5.43E3",
    "5.43E3",
  ],
  [
    543000,
    "543E3",
    "5.43E5",
  ],
  [
    543211.1,
    "543.211E3",
    "5.432E5",
  ],
  [
    -Infinity,
    "-∞",
    "-∞",
  ],
  [
    Infinity,
    "∞",
    "∞",
  ],
  [
    NaN,
    "NaN",
    "NaN",
  ],
];

for (const [number, engineering, scientific] of tests) {
  const nfEngineering = (new Intl.NumberFormat("en-US", { notation: "engineering" }));
  assert.sameValue(nfEngineering.format(number), engineering);
  const nfScientific = (new Intl.NumberFormat("en-US", { notation: "scientific" }));
  assert.sameValue(nfScientific.format(number), scientific);
}

