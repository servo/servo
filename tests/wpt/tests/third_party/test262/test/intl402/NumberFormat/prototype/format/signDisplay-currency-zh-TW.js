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
    "(US$987.00)",
    "(US$0.00)",
    "(US$0.00)",
    "US$0.00",
    "US$0.00",
    "US$987.00",
  ],
  [
    "always",
    "(US$987.00)",
    "(US$0.00)",
    "(US$0.00)",
    "+US$0.00",
    "+US$0.00",
    "+US$987.00",
  ],
  [
    "never",
    "US$987.00",
    "US$0.00",
    "US$0.00",
    "US$0.00",
    "US$0.00",
    "US$987.00",
  ],
  [
    "exceptZero",
    "(US$987.00)",
    "US$0.00",
    "US$0.00",
    "US$0.00",
    "US$0.00",
    "+US$987.00",
  ],
];

for (const [signDisplay, negative, negativeNearZero, negativeZero, zero, positiveNearZero, positive] of tests) {
  const nf = new Intl.NumberFormat("zh-TW", { style: "currency", currency: "USD", currencySign: "accounting", signDisplay });
  assert.sameValue(nf.format(-987), negative);
  assert.sameValue(nf.format(-0.0001), negativeNearZero);
  assert.sameValue(nf.format(-0), negativeZero);
  assert.sameValue(nf.format(0), zero);
  assert.sameValue(nf.format(0.0001), positiveNearZero);
  assert.sameValue(nf.format(987), positive);
}

