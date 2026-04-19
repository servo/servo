// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [zh-TW]
features: [Intl.NumberFormat-unified]
---*/


const tests = [
  [
    "auto",
    "-∞",
    "-987",
    "-0",
    "-0",
    "0",
    "0",
    "987",
    "∞",
    "非數值",
  ],
  [
    "always",
    "-∞",
    "-987",
    "-0",
    "-0",
    "+0",
    "+0",
    "+987",
    "+∞",
    "+非數值",
  ],
  [
    "never",
    "∞",
    "987",
    "0",
    "0",
    "0",
    "0",
    "987",
    "∞",
    "非數值",
  ],
  [
    "exceptZero",
    "-∞",
    "-987",
    "0",
    "0",
    "0",
    "0",
    "+987",
    "+∞",
    "非數值",
  ],
];

for (const [signDisplay, ...expected] of tests) {
  const nf = new Intl.NumberFormat("zh-TW", {signDisplay});
  assert.sameValue(nf.format(-Infinity), expected[0], `-Infinity (${signDisplay})`);
  assert.sameValue(nf.format(-987), expected[1], `-987 (${signDisplay})`);
  assert.sameValue(nf.format(-0.0001), expected[2], `-0.0001 (${signDisplay})`);
  assert.sameValue(nf.format(-0), expected[3], `-0 (${signDisplay})`);
  assert.sameValue(nf.format(0), expected[4], `0 (${signDisplay})`);
  assert.sameValue(nf.format(0.0001), expected[5], `0.0001 (${signDisplay})`);
  assert.sameValue(nf.format(987), expected[6], `987 (${signDisplay})`);
  assert.sameValue(nf.format(Infinity), expected[7], `Infinity (${signDisplay})`);
  assert.sameValue(nf.format(NaN), expected[8], `NaN (${signDisplay})`);
}

