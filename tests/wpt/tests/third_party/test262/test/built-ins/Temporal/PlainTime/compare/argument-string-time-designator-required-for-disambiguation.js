// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: ISO 8601 time designator "T" required in cases of ambiguity
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const midnight = new Temporal.PlainTime();

TemporalHelpers.ISO.plainTimeStringsAmbiguous().forEach((string) => {
  let arg = string;
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(arg, midnight),
    `'${arg}' is ambiguous and requires T prefix (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(midnight, arg),
    `'${arg}' is ambiguous and requires T prefix (second argument)`
  );
  // The same string with a T prefix should not throw:
  arg = `T${string}`;
  Temporal.PlainTime.compare(arg, midnight);
  Temporal.PlainTime.compare(midnight, arg);

  arg = ` ${string}`;
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(arg, midnight),
    `space is not accepted as a substitute for T prefix (first argument): '${arg}'`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.compare(midnight, arg),
    `space is not accepted as a substitute for T prefix (second argument): '${arg}'`
  );
});

// None of these should throw without a T prefix, because they are unambiguously time strings:
TemporalHelpers.ISO.plainTimeStringsUnambiguous().forEach((arg) => {
  Temporal.PlainTime.compare(arg, midnight);
  Temporal.PlainTime.compare(midnight, arg);
});
