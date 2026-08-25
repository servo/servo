// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [ja-JP]
features: [Intl.NumberFormat-v3]
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

const nf = new Intl.NumberFormat("ja-JP", { style: "currency", currency: "USD", currencySign: "accounting", signDisplay: "negative" });

verifyFormatParts(
  nf.formatToParts(-987),
  [{"type":"literal","value":"("},{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"},{"type":"literal","value":")"}],
  "negative"
);
verifyFormatParts(
  nf.formatToParts(-0.0001),
  [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  "negativeNearZero"
);
verifyFormatParts(
  nf.formatToParts(-0),
  [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  "negativeZero"
);
verifyFormatParts(
  nf.formatToParts(0),
  [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  "zero"
);
verifyFormatParts(
  nf.formatToParts(0.0001),
  [{"type":"currency","value":"$"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  "positiveNearZero"
);
verifyFormatParts(
  nf.formatToParts(987),
  [{"type":"currency","value":"$"},{"type":"integer","value":"987"},{"type":"decimal","value":"."},{"type":"fraction","value":"00"}],
  "positive"
);
