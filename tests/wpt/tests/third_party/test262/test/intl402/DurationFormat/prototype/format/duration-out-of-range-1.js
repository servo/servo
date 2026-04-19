// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  IsValidDurationRecord rejects too large "years", "months", and "weeks" values.
info: |
  Intl.DurationFormat.prototype.format ( duration )
  ...
  3. Let record be ? ToDurationRecord(duration).
  ...

  ToDurationRecord ( input )
  ...
  24. If IsValidDurationRecord(result) is false, throw a RangeError exception.
  ...

  IsValidDurationRecord ( record )
  ...
  6. If abs(years) ≥ 2^32, return false.
  7. If abs(months) ≥ 2^32, return false.
  8. If abs(weeks) ≥ 2^32, return false.
  ...

features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat();

const units = [
  "years",
  "months",
  "weeks",
];

const invalidValues = [
  2**32,
  2**32 + 1,
  Number.MAX_SAFE_INTEGER,
  Number.MAX_VALUE,
];

const validValues = [
  2**32 - 1,
];

for (let unit of units) {
  for (let value of invalidValues) {
    let positive = {[unit]: value};
    assert.throws(
      RangeError,
      () => df.format(positive),
      `Duration "${unit}" throws when value is ${value}`
    );

    // Also test with flipped sign.
    let negative = {[unit]: -value};
    assert.throws(
      RangeError,
      () => df.format(negative),
      `Duration "${unit}" throws when value is ${-value}`
    );
  }

  for (let value of validValues) {
    // We don't care about the exact contents of the returned string, the call
    // just shouldn't throw an exception.
    let positive = {[unit]: value};
    assert.sameValue(
      typeof df.format(positive),
      "string",
      `Duration "${unit}" doesn't throw when value is ${value}`
    );

    // Also test with flipped sign.
    let negative = {[unit]: -value};
    assert.sameValue(
      typeof df.format(negative),
      "string",
      `Duration "${unit}" doesn't throw when value is ${-value}`
    );
  }
}
