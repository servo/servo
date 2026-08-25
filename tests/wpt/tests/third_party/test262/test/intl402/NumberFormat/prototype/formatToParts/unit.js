// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the unit style.
features: [Intl.NumberFormat-unified]
---*/

const numbers = [-987, -0.001, -0, 0, 0.001, 987];
const displays = [
  "short",
  "narrow",
  "long",
];

for (const unitDisplay of displays) {
  const nf = new Intl.NumberFormat("en-US", { style: "unit", unit: "meter", unitDisplay });
  for (const number of numbers) {
    const result = nf.formatToParts(number);
    assert.sameValue(result.map(({ value }) => value).join(""), nf.format(number));
    assert.sameValue(result.some(({ type }) => type === "unit"), true);
  }
}

