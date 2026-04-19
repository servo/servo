// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Test behaviour around DST boundaries with various combinations of values for
  the options offset and disambiguation, when the argument is a property bag.
features: [Temporal]
---*/

// Non existent zoned date time - Spring DST
const DSTStart = {
  year: 2000,
  month: 4,
  day: 2,
  hour: 2,
  minute: 30,
  timeZone: "America/Vancouver"
};

// Uses disambiguation if offset option is set to "ignore".
let offset = "ignore";
let zdt = Temporal.ZonedDateTime.from(DSTStart, {
  offset,
  disambiguation: "compatible"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset: ignore and disambiguation: compatible");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset: ignore and disambiguation: compatible");

zdt = Temporal.ZonedDateTime.from(DSTStart, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when offset: ignore and disambiguation: earlier");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when offset: ignore and disambiguation: earlier");

zdt = Temporal.ZonedDateTime.from(DSTStart, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset: ignore and disambiguation: later");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset: ignore and disambiguation: later");

assert.throws(RangeError, () => Temporal.ZonedDateTime.from(DSTStart, {
  offset,
  disambiguation: "reject"
}), "Throws when offset: ignore and disambiguation: reject");

// Uses disambiguation if the property bag's offset is wrong and the offset
// option is set to "prefer".
const DSTStartWithWrongOffset = {
  ...DSTStart,
  offset: "-23:59"
};
offset = "prefer";

zdt = Temporal.ZonedDateTime.from(DSTStartWithWrongOffset, {
  offset,
  disambiguation: "compatible"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset is wrong, option offset: prefer, and disambiguation: compatible");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset is wrong, option offset: prefer, and disambiguation: compatible");

zdt = Temporal.ZonedDateTime.from(DSTStartWithWrongOffset, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when offset is wrong, option offset: prefer, and disambiguation: earlier");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when offset is wrong, option offset: prefer, and disambiguation: earlier");

zdt = Temporal.ZonedDateTime.from(DSTStartWithWrongOffset, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: prefer, and disambiguation: later");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: prefer, and disambiguation: later");

assert.throws(RangeError, () => Temporal.ZonedDateTime.from(DSTStartWithWrongOffset, {
  offset,
  disambiguation: "reject"
}), "Throws when offset is wrong, option offset: prefer, and disambiguation: reject");
