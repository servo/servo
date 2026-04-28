// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: ISO strings at the edges of the representable range
features: [Temporal]
---*/

// offset: "use" takes the exact time with the given UTC offset, and so doesn't
// need to compute the UTC epoch nanoseconds at the wall-clock time.
// offset: "ignore" takes the UTC offset from the time zone, which in the case
// of an offset time zone also doesn't need to compute the UTC epoch nanoseconds
// at the wall-clock time.

const validStringsForOffsetUseIgnore = [
  "-271821-04-20T00:00Z[UTC]",
  "-271821-04-19T23:00-01:00[-01:00]",
  "-271821-04-19T00:01-23:59[-23:59]",
  "+275760-09-13T00:00Z[UTC]",
  "+275760-09-13T01:00+01:00[+01:00]",
  "+275760-09-13T23:59+23:59[+23:59]",
];

for (const offset of ["use", "ignore"]) {
  for (const arg of validStringsForOffsetUseIgnore) {
    Temporal.ZonedDateTime.from(arg, { offset });
  }
}

// Other values for offset need to compute the UTC epoch nanoseconds at the
// wall-clock time, so that can't be outside the representable range even if
// the signified exact time is inside.

const validStringsForOffsetPreferReject = [
  "-271821-04-20T00:00Z[UTC]",
  "+275760-09-13T00:00Z[UTC]",
  "+275760-09-13T01:00+01:00[+01:00]",
  "+275760-09-13T23:59+23:59[+23:59]",
];

const invalidStringsForOffsetPreferReject = [
  "-271821-04-19T23:00-01:00[-01:00]",
  "-271821-04-19T00:00:01-23:59[-23:59]",
];

for (const offset of ["prefer", "reject"]) {
  for (const arg of validStringsForOffsetPreferReject) {
    Temporal.ZonedDateTime.from(arg, { offset });
  }

  for (const arg of invalidStringsForOffsetPreferReject) {
    assert.throws(
      RangeError,
      () => Temporal.ZonedDateTime.from(arg, { offset }),
      `wall-clock time of "${arg}" is outside the representable range of ZonedDateTime (offset=${offset})`
    );
  }
}

const invalidStrings = [
  "-271821-04-19T23:59:59.999999999Z[UTC]",
  "-271821-04-19T23:00-00:59[-00:59]",
  "-271821-04-19T00:00:00-23:59[-23:59]",
  "+275760-09-13T00:00:00.000000001Z[UTC]",
  "+275760-09-13T01:00+00:59[+00:59]",
  "+275760-09-14T00:00+23:59[+23:59]",
];

for (const offset of ["use", "ignore", "prefer", "reject"]) {
  for (const arg of invalidStrings) {
    assert.throws(
      RangeError,
      () => Temporal.ZonedDateTime.from(arg, { offset }),
      `"${arg}" is outside the representable range of ZonedDateTime (offset=${offset})`
    );
  }
}
