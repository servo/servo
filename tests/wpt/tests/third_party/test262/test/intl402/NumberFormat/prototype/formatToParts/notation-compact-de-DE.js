// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Checks handling of the compactDisplay option to the NumberFormat constructor.
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
    987654321,
    [{"type":"integer","value":"988"},{"type":"literal","value":"\u00a0"},{"type":"compact","value":"Mio."}],
    [{"type":"integer","value":"988"},{"type":"literal","value":" "},{"type":"compact","value":"Millionen"}],
  ],
  [
    98765432,
    [{"type":"integer","value":"99"},{"type":"literal","value":"\u00a0"},{"type":"compact","value":"Mio."}],
    [{"type":"integer","value":"99"},{"type":"literal","value":" "},{"type":"compact","value":"Millionen"}],
  ],
  [
    98765,
    [{"type":"integer","value":"98"},{"type":"group","value":"."},{"type":"integer","value":"765"}],
    [{"type":"integer","value":"99"},{"type":"literal","value":" "},{"type":"compact","value":"Tausend"}],
  ],
  [
    9876,
    [{"type":"integer","value":"9876"}],
    [{"type":"integer","value":"9"},{"type":"decimal","value":","},{"type":"fraction","value":"9"},{"type":"literal","value":" "},{"type":"compact","value":"Tausend"}],
  ],
  [
    159,
    [{"type":"integer","value":"159"}],
  ],
  [
    15.9,
    [{"type":"integer","value":"16"}],
  ],
  [
    1.59,
    [{"type":"integer","value":"1"},{"type":"decimal","value":","},{"type":"fraction","value":"6"}],
  ],
  [
    0.159,
    [{"type":"integer","value":"0"},{"type":"decimal","value":","},{"type":"fraction","value":"16"}],
  ],
  [
    0.0159,
    [{"type":"integer","value":"0"},{"type":"decimal","value":","},{"type":"fraction","value":"016"}],
  ],
  [
    0.00159,
    [{"type":"integer","value":"0"},{"type":"decimal","value":","},{"type":"fraction","value":"0016"}],
  ],
  [
    -Infinity,
    [{"type":"minusSign","value":"-"},{"type":"infinity","value":"∞"}],
  ],
  [
    Infinity,
    [{"type":"infinity","value":"∞"}],
  ],
  [
    NaN,
    [{"type":"nan","value":"NaN"}],
  ],
];

for (const [number, short, long = short] of tests) {
  const nfShort = new Intl.NumberFormat("de-DE", {
    notation: "compact",
    compactDisplay: "short",
  });
  verifyFormatParts(nfShort.formatToParts(number), short, `Compact short: ${number}`);

  const nfLong = new Intl.NumberFormat("de-DE", {
    notation: "compact",
    compactDisplay: "long",
  });
  verifyFormatParts(nfLong.formatToParts(number), long, `Compact long: ${number}`);
}
