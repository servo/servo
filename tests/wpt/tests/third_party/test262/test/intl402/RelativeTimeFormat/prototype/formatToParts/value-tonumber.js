// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the handling of non-number value arguments to Intl.RelativeTimeFormat.prototype.format().
info: |
    Intl.RelativeTimeFormat.prototype.format( value, unit )

    3. Let value be ? ToNumber(value).

features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.format, "function", "format should be supported");

const values = [
  [null, 0],

  [true, 1],
  [false, 0],

  ["5", 5],
  ["-5", -5],
  ["0", 0],
  ["-0", -0],
  ["  6  ", 6],

  [{ toString() { return 7; } }, 7, "object with toString"],
  [{ valueOf() { return 7; } }, 7, "object with valueOf"],
];

for (const [input, number, name = String(input)] of values) {
  const actual = rtf.formatToParts(input, "second");
  const expected = rtf.formatToParts(number, "second");

  assert.sameValue(actual.length, expected.length,
                   `Should treat ${name} as ${number}: length`);

  for (let i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type,
                     `Should treat ${name} as ${number}: [${i}].type`);
    assert.sameValue(actual[i].value, expected[i].value,
                     `Should treat ${name} as ${number}: [${i}].value`);
    assert.sameValue(actual[i].unit, expected[i].unit,
                     `Should treat ${name} as ${number}: [${i}].unit`);
  }
}
