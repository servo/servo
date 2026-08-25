// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the engineering and scientific notations.
locale: [de-DE]
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
    0.000345,
    [{"type":"integer","value":"345"},{"type":"exponentSeparator","value":"E"},{"type":"exponentMinusSign","value":"-"},{"type":"exponentInteger","value":"6"}],
    [{"type":"integer","value":"3"},{"type":"decimal","value":","},{"type":"fraction","value":"45"},{"type":"exponentSeparator","value":"E"},{"type":"exponentMinusSign","value":"-"},{"type":"exponentInteger","value":"4"}],
  ],
  [
    0.345,
    [{"type":"integer","value":"345"},{"type":"exponentSeparator","value":"E"},{"type":"exponentMinusSign","value":"-"},{"type":"exponentInteger","value":"3"}],
    [{"type":"integer","value":"3"},{"type":"decimal","value":","},{"type":"fraction","value":"45"},{"type":"exponentSeparator","value":"E"},{"type":"exponentMinusSign","value":"-"},{"type":"exponentInteger","value":"1"}],
  ],
  [
    3.45,
    [{"type":"integer","value":"3"},{"type":"decimal","value":","},{"type":"fraction","value":"45"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"0"}],
    [{"type":"integer","value":"3"},{"type":"decimal","value":","},{"type":"fraction","value":"45"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"0"}],
  ],
  [
    34.5,
    [{"type":"integer","value":"34"},{"type":"decimal","value":","},{"type":"fraction","value":"5"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"0"}],
    [{"type":"integer","value":"3"},{"type":"decimal","value":","},{"type":"fraction","value":"45"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"1"}],
  ],
  [
    543,
    [{"type":"integer","value":"543"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"0"}],
    [{"type":"integer","value":"5"},{"type":"decimal","value":","},{"type":"fraction","value":"43"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"2"}],
  ],
  [
    5430,
    [{"type":"integer","value":"5"},{"type":"decimal","value":","},{"type":"fraction","value":"43"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"3"}],
    [{"type":"integer","value":"5"},{"type":"decimal","value":","},{"type":"fraction","value":"43"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"3"}],
  ],
  [
    543000,
    [{"type":"integer","value":"543"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"3"}],
    [{"type":"integer","value":"5"},{"type":"decimal","value":","},{"type":"fraction","value":"43"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"5"}],
  ],
  [
    543211.1,
    [{"type":"integer","value":"543"},{"type":"decimal","value":","},{"type":"fraction","value":"211"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"3"}],
    [{"type":"integer","value":"5"},{"type":"decimal","value":","},{"type":"fraction","value":"432"},{"type":"exponentSeparator","value":"E"},{"type":"exponentInteger","value":"5"}],
  ],
  [
    -Infinity,
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
  ],
  [
    Infinity,
    [{"type":"infinity","value":"∞"}],
    [{"type":"infinity","value":"∞"}],
  ],
  [
    NaN,
    [{"type":"nan","value":"NaN"}],
    [{"type":"nan","value":"NaN"}],
  ],
];

for (const [number, engineering, scientific] of tests) {
  const nfEngineering = (new Intl.NumberFormat("de-DE", { notation: "engineering" }));
  verifyFormatParts(nfEngineering.formatToParts(number), engineering, `${number} - engineering`);
  const nfScientific = (new Intl.NumberFormat("de-DE", { notation: "scientific" }));
  verifyFormatParts(nfScientific.formatToParts(number), scientific, `${number} - scientific`);
}

