// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Fuzzy matching behaviour with UTC offsets in ISO 8601 strings with named time zones and offset option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

["use", "ignore", "prefer", "reject"].forEach((offset) => {
  const result = Temporal.ZonedDateTime.from("1970-01-01T12:00-00:44:30[Africa/Monrovia]", { offset });
  assert.sameValue(result.epochNanoseconds, 45870_000_000_000n, `accepts the exact offset string (offset: ${offset})`);
  assert.sameValue(result.offset, "-00:44:30", "offset property is correct");
});

["use", "ignore", "prefer", "reject"].forEach((offset) => {
  const result = Temporal.ZonedDateTime.from("1970-01-01T12:00-00:44:30.000000000[Africa/Monrovia]", { offset });
  assert.sameValue(
    result.epochNanoseconds,
    45870_000_000_000n,
    `accepts trailing zeroes after ISO string offset (offset: ${offset})`
  );
  assert.sameValue(result.offset, "-00:44:30", "offset property removes trailing zeroes from input");
});

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from("1970-01-01T00:00-00:44:30[-00:45]", { offset: "reject" }),
  "minute rounding not supported for offset time zones"
);

const str = "1970-01-01T12:00-00:45[Africa/Monrovia]";

["ignore", "prefer", "reject"].forEach((offset) => {
  const result = Temporal.ZonedDateTime.from(str, { offset });
  assert.sameValue(
    result.epochNanoseconds,
    45870_000_000_000n,
    `accepts the offset string rounded to minutes (offset=${offset})`
  );
  assert.sameValue(result.offset, "-00:44:30", "offset property is still the full precision");
  TemporalHelpers.assertPlainDateTime(
    result.toPlainDateTime(),
    1970,
    1,
    "M01",
    1,
    12,
    0,
    0,
    0,
    0,
    0,
    "wall time is preserved"
  );
});

const result = Temporal.ZonedDateTime.from(str, { offset: "use" });
assert.sameValue(
  result.epochNanoseconds,
  45900_000_000_000n,
  "prioritizes the offset string with HH:MM precision when offset=use"
);
assert.sameValue(result.offset, "-00:44:30", "offset property is still the full precision");
TemporalHelpers.assertPlainDateTime(
  result.toPlainDateTime(),
  1970,
  1,
  "M01",
  1,
  12,
  0,
  30,
  0,
  0,
  0,
  "wall time is shifted by the difference between exact and rounded offset"
);

const wrongSeconds = "1970-01-01T12-00:44:40[Africa/Monrovia]";
const roundedSeconds = "1970-01-01T12-00:45:00[Africa/Monrovia]";

const useResultWrongSeconds = Temporal.ZonedDateTime.from(wrongSeconds, { offset: "use" });
assert.sameValue(
  useResultWrongSeconds.epochNanoseconds,
  45880_000_000_000n,
  "uses the wrong offset with HH:MM:SS precision when offset=use"
);
assert.sameValue(useResultWrongSeconds.offset, "-00:44:30", "offset property is still the full precision");
TemporalHelpers.assertPlainDateTime(
  useResultWrongSeconds.toPlainDateTime(),
  1970,
  1,
  "M01",
  1,
  12,
  0,
  10,
  0,
  0,
  0,
  "wall time is shifted by the difference between exact and given offset"
);


const useResultRoundedSeconds = Temporal.ZonedDateTime.from(roundedSeconds, { offset: "use" });
assert.sameValue(
  useResultRoundedSeconds.epochNanoseconds,
  45900_000_000_000n,
  "uses the rounded offset with HH:MM:SS precision when offset=use"
);
assert.sameValue(useResultRoundedSeconds.offset, "-00:44:30", "offset property is still the full precision");
TemporalHelpers.assertPlainDateTime(
  useResultRoundedSeconds.toPlainDateTime(),
  1970,
  1,
  "M01",
  1,
  12,
  0,
  30,
  0,
  0,
  0,
  "wall time is shifted by the difference between exact and given offset"
);

["ignore", "prefer"].forEach((offset) => {
  const resultWrongSeconds = Temporal.ZonedDateTime.from(wrongSeconds, { offset });
  assert.sameValue(
    resultWrongSeconds.epochNanoseconds,
    45870_000_000_000n,
    `does not use the offset string with wrong :SS (offset=${offset})`
  );
  assert.sameValue(resultWrongSeconds.offset, "-00:44:30", "offset property is still the full precision");
  TemporalHelpers.assertPlainDateTime(
    resultWrongSeconds.toPlainDateTime(),
    1970,
    1,
    "M01",
    1,
    12,
    0,
    0,
    0,
    0,
    0,
    "wall time is preserved"
  );

  const resultRoundedSeconds = Temporal.ZonedDateTime.from(roundedSeconds, { offset });
  assert.sameValue(
    resultRoundedSeconds.epochNanoseconds,
    45870_000_000_000n,
    `does not use the offset string with rounded HH:MM:SS (offset=${offset})`
  );
  assert.sameValue(resultRoundedSeconds.offset, "-00:44:30", "offset property is still the full precision");
  TemporalHelpers.assertPlainDateTime(
    resultRoundedSeconds.toPlainDateTime(),
    1970,
    1,
    "M01",
    1,
    12,
    0,
    0,
    0,
    0,
    0,
    "wall time is preserved"
  );
});

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from(wrongSeconds, { offset: "reject" }),
  "wrong :SS not accepted in string offset (offset=reject)"
);

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from(roundedSeconds, { offset: "reject" }),
  "rounded HH:MM:SS not accepted in string offset (offset=reject)"
);

const properties = { year: 1970, month: 1, day: 1, hour: 12, offset: "-00:45", timeZone: "Africa/Monrovia" };

["ignore", "prefer"].forEach((offset) => {
  const result = Temporal.ZonedDateTime.from(properties, { offset });
  assert.sameValue(
    result.epochNanoseconds,
    45870_000_000_000n,
    `no fuzzy matching is done on offset in property bag (offset=${offset})`
  );
});

const result2 = Temporal.ZonedDateTime.from(properties, { offset: "use" });
assert.sameValue(
  result2.epochNanoseconds,
  45900_000_000_000n,
  "no fuzzy matching is done on offset in property bag (offset=use)"
);

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from(properties, { offset: "reject" }),
  "no fuzzy matching is done on offset in property bag (offset=reject)"
);

// Pacific/Niue edge case

const reference = -543069621_000_000_000n;

assert.sameValue(
  Temporal.ZonedDateTime.from("1952-10-15T23:59:59-11:19:40[Pacific/Niue]").epochNanoseconds,
  reference,
  "-11:19:40 is accepted as -11:19:40 in Pacific/Niue edge case"
);
assert.sameValue(
  Temporal.ZonedDateTime.from("1952-10-15T23:59:59-11:20[Pacific/Niue]").epochNanoseconds,
  reference,
  "-11:20 matches the first candidate -11:19:40 in the Pacific/Niue edge case"
);
assert.sameValue(
  Temporal.ZonedDateTime.from("1952-10-15T23:59:59-11:20:00[Pacific/Niue]").epochNanoseconds,
  reference + 20_000_000_000n,
  "-11:20:00 is accepted as -11:20:00 in the Pacific/Niue edge case"
);
assert.throws(
  RangeError, () => Temporal.ZonedDateTime.from("1952-10-15T23:59:59-11:19:50[Pacific/Niue]"),
  "wrong :SS not accepted in the Pacific/Niue edge case"
);
