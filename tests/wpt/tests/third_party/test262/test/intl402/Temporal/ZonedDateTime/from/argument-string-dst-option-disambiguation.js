// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Test behaviour around DST boundaries with the option disambiguation set, when
  the argument is a string.
features: [Temporal]
---*/

// Ambiguous zoned date time - Fall DST
const DSTEnd = "2019-02-16T23:45[America/Sao_Paulo]";
let zdt = Temporal.ZonedDateTime.from(DSTEnd, { disambiguation: "compatible" });
assert.sameValue(
  zdt.offset,
  "-02:00",
  "Offset result when option disambiguation: compatible ambiguous time");

zdt = Temporal.ZonedDateTime.from(DSTEnd, { disambiguation: "earlier" });
assert.sameValue(
  zdt.offset,
  "-02:00",
  "Offset result when option disambiguation: earlier, ambiguous time");

zdt = Temporal.ZonedDateTime.from(DSTEnd, { disambiguation: "later" });
assert.sameValue(
  zdt.offset,
  "-03:00",
  "Offset result when option disambiguation: later, ambiguous time");

assert.throws(RangeError, () =>
  Temporal.ZonedDateTime.from(DSTEnd, { disambiguation: "reject" }),
  "Throws when option disambiguation: reject, ambiguous time");

// Zoned date time in non existent time - Spring DST
const DSTStart = "2020-03-08T02:30[America/Los_Angeles]";
zdt = Temporal.ZonedDateTime.from(DSTStart, { disambiguation: "compatible" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option disambiguation: compatible, non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option disambiguation: compatible, non existent time");

zdt = Temporal.ZonedDateTime.from(DSTStart, { disambiguation: "earlier" });
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option disambiguation: earlier, non existent time");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when option disambiguation: earlier, non existent time");

zdt = Temporal.ZonedDateTime.from(DSTStart, { disambiguation: "later" });
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option disambiguation: later, non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option disambiguation: later, non existent time");

assert.throws(RangeError, () =>
  Temporal.ZonedDateTime.from(DSTStart, { disambiguation: "reject" }),
  "Throws when option disambiguation: reject, non existent time");
