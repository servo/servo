// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the unit style.
locale: [ko-KR]
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
    -987,
    {
      "short":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"987"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"minusSign","value":"-"},{"type":"integer","value":"987"},{"type":"unit","value":"킬로미터"}],
    }
  ],
  [
    -0.001,
    {
      "short":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"킬로미터"}],
    }
  ],
  [
    -0,
    {
      "short":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"minusSign","value":"-"},{"type":"integer","value":"0"},{"type":"unit","value":"킬로미터"}],
    }
  ],
  [
    0,
    {
      "short":
        [{"type":"integer","value":"0"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"integer","value":"0"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"integer","value":"0"},{"type":"unit","value":"킬로미터"}],
    }
  ],
  [
    0.001,
    {
      "short":
        [{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"integer","value":"0"},{"type":"decimal","value":"."},{"type":"fraction","value":"001"},{"type":"unit","value":"킬로미터"}],
    }
  ],
  [
    987,
    {
      "short":
        [{"type":"integer","value":"987"},{"type":"unit","value":"km/h"}],
      "narrow":
        [{"type":"integer","value":"987"},{"type":"unit","value":"km/h"}],
      "long":
        [{"type":"unit","value":"시속"},{"type":"literal","value":" "},{"type":"integer","value":"987"},{"type":"unit","value":"킬로미터"}],
    }
  ],
];

for (const [number, expectedData] of tests) {
  for (const [unitDisplay, expected] of Object.entries(expectedData)) {
    const nf = new Intl.NumberFormat("ko-KR", { style: "unit", unit: "kilometer-per-hour", unitDisplay });
    verifyFormatParts(nf.formatToParts(number), expected);
  }
}

