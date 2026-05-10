// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: ISO 8601 time designator "T" required in cases of ambiguity
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

TemporalHelpers.ISO.plainTimeStringsAmbiguous().forEach((string) => {
  let arg = string;
  assert.throws(
    RangeError,
    () => instance.since(arg),
    `'${arg}' is ambiguous and requires T prefix`
  );
  // The same string with a T prefix should not throw:
  arg = `T${string}`;
  instance.since(arg);

  arg = ` ${string}`;
  assert.throws(
    RangeError,
    () => instance.since(arg),
    `space is not accepted as a substitute for T prefix: '${arg}'`
  );
});

// None of these should throw without a T prefix, because they are unambiguously time strings:
TemporalHelpers.ISO.plainTimeStringsUnambiguous().forEach(
  (arg) => instance.since(arg));
