// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [en-US]
features: [Intl.NumberFormat-unified]
---*/

function verifyFormatParts(actual, expected, message) {
  assert.sameValue(Array.isArray(expected), true, `${message}: expected is Array`);
  assert.sameValue(Array.isArray(actual), true, `${message}: actual is Array`);
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (let i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type, `${message}: parts[${i}].type`);
    assert.sameValue(actual[i].value, expected[i].value, `${message}: parts[${i}].value`);
  }
}

const tests = [
  [
    "auto",
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  ],
  [
    "always",
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"plusSign","value":"+"},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"plusSign","value":"+"},{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"plusSign","value":"+"},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  ],
  [
    "never",
    [{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  ],
  [
    "exceptZero",
    [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
    [{"type":"plusSign","value":"+"},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  ],
];

for (const [signDisplay, negative, negativeNearZero, negativeZero, zero, positiveNearZero, positive] of tests) {
  const nf = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD", currencySign: "accounting", signDisplay });
  verifyFormatParts(nf.formatToParts(-987), negative, "negative");
  verifyFormatParts(nf.formatToParts(-0.0001), negativeNearZero, "negativeNearZero");
  verifyFormatParts(nf.formatToParts(-0), negativeZero, "negativeZero");
  verifyFormatParts(nf.formatToParts(0), zero, "zero");
  verifyFormatParts(nf.formatToParts(0.0001), positiveNearZero, "positiveNearZero");
  verifyFormatParts(nf.formatToParts(987), positive, "positive");
}

