// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [ja-JP]
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
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"987"}],
    [{"type":"infinity","value":"∞"}],
    [{"type":"nan","value":"NaN"}],
  ],
  [
    "always",
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"}],
    [{"type":"plusSign","value":"+"},{"type":"integer","value":"0"}],
    [{"type":"plusSign","value":"+"},{"type":"integer","value":"0"}],
    [{"type":"plusSign","value":"+"},{"type":"integer","value":"987"}],
    [{"type":"plusSign","value":"+"},{"type":"infinity","value":"∞"}],
    [{"type":"plusSign","value":"+"},{"type":"nan","value":"NaN"}],
  ],
  [
    "never",
    [{"type":"infinity","value":"∞"}],
    [{"type":"integer","value":"987"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"987"}],
    [{"type":"infinity","value":"∞"}],
    [{"type":"nan","value":"NaN"}],
  ],
  [
    "exceptZero",
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
    [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"integer","value":"0"}],
    [{"type":"plusSign","value":"+"},{"type":"integer","value":"987"}],
    [{"type":"plusSign","value":"+"},{"type":"infinity","value":"∞"}],
    [{"type":"nan","value":"NaN"}],
  ],
];

for (const [signDisplay, ...expected] of tests) {
  const nf = new Intl.NumberFormat("ja-JP", {signDisplay});
  verifyFormatParts(nf.formatToParts(-Infinity), expected[0], `-Infinity (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(-987), expected[1], `-987 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(-0.0001), expected[2], `-0.0001 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(-0), expected[3], `-0 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(0), expected[4], `0 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(0.0001), expected[5], `0.0001 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(987), expected[6], `987 (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(Infinity), expected[7], `Infinity (${signDisplay})`);
  verifyFormatParts(nf.formatToParts(NaN), expected[8], `NaN (${signDisplay})`);
}

