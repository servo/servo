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
  assert.sameValue(rtf.format(input, "second"), rtf.format(number, "second"),
                   `Should treat ${name} as ${number}`)
}
