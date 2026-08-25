// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: ISO 8601 time designator "T" required in cases of ambiguity
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

TemporalHelpers.ISO.plainTimeStringsAmbiguous().forEach((string) => {
  let arg = string;
  assert.throws(
    RangeError,
    () => instance.withPlainTime(arg),
    `'${arg}' is ambiguous and requires T prefix`
  );
  // The same string with a T prefix should not throw:
  arg = `T${string}`;
  instance.withPlainTime(arg);

  arg = ` ${string}`;
  assert.throws(
    RangeError,
    () => instance.withPlainTime(arg),
    `space is not accepted as a substitute for T prefix: '${arg}'`
  );
});

// None of these should throw without a T prefix, because they are unambiguously time strings:
TemporalHelpers.ISO.plainTimeStringsUnambiguous().forEach(
  (arg) => instance.withPlainTime(arg));
