// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tolocalestring
description: >
    Tests that BigInt.prototype.toLocaleString produces the same
    results as Intl.BigIntFormat.
features: [BigInt]
---*/

var inputs = [
  0n,
  -0n,
  1n,
  -1n,
  123n,
  -123n,
  12345n,
  -12345n,
  12344501000000000000000000000000000n,
  -12344501000000000000000000000000000n,
];
var localesInputs = [undefined, ["de"], ["th-u-nu-thai"], ["en"], ["ja-u-nu-jpanfin"], ["ar-u-nu-arab"]];
var optionsInputs = [
  undefined,
  {style: "percent"},
  {style: "currency", currency: "EUR", currencyDisplay: "symbol"},
  {style: "currency", currency: "IQD", currencyDisplay: "symbol"},
  {style: "currency", currency: "KMF", currencyDisplay: "symbol"},
  {style: "currency", currency: "CLF", currencyDisplay: "symbol"},
  {useGrouping: false, minimumIntegerDigits: 3, minimumFractionDigits: 1, maximumFractionDigits: 3}
];

for (const locales of localesInputs) {
  for (const options of optionsInputs) {
    const optionsString = options ? JSON.stringify(options) : String(options);
    const referenceNumberFormat = new Intl.NumberFormat(locales, options);
    for (const input of inputs) {
      const referenceFormatted = referenceNumberFormat.format(input);
      const formatted = input.toLocaleString(locales, options);
      assert.sameValue(formatted, referenceFormatted,
                       `(Testing with locales ${locales}; options ${optionsString}.)`);
    }
  }
}
