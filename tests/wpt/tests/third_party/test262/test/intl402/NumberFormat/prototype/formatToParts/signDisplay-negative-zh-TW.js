// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [zh-TW]
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

const signDisplay = "negative";
const nf = new Intl.NumberFormat("zh-TW", {signDisplay: "negative"});

verifyFormatParts(nf.formatToParts(-Infinity), [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}], `-Infinity (${signDisplay})`);
verifyFormatParts(nf.formatToParts(-987), [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"}], `-987 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(-0.0001), [{"type":"integer","value":"0"}], `-0.0001 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(-0), [{"type":"integer","value":"0"}], `-0 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(0), [{"type":"integer","value":"0"}], `0 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(0.0001), [{"type":"integer","value":"0"}], `0.0001 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(987), [{"type":"integer","value":"987"}], `987 (${signDisplay})`);
verifyFormatParts(nf.formatToParts(Infinity), [{"type":"infinity","value":"∞"}], `Infinity (${signDisplay})`);
verifyFormatParts(nf.formatToParts(NaN), [{"type":"nan","value":"非數值"}], `NaN (${signDisplay})`);
