// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: >
  Test behaviour around DST boundaries with various combinations of values for
  the options offset and disambiguation.
features: [Temporal]
---*/

const dstStartDay = Temporal.PlainDateTime.from("2000-04-02T12:00:01").toZonedDateTime("America/Vancouver");
const dstEndDay = Temporal.PlainDateTime.from("2000-10-29T12:00:01").toZonedDateTime("America/Vancouver");
const oneThirty = {
  hour: 1,
  minute: 30
};
const twoThirty = {
  hour: 2,
  minute: 30
};

let offset = "ignore";
// disambiguation: compatible, skipped wall time
let zdt = dstStartDay.with(twoThirty, {
  offset,
  disambiguation: "compatible"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore and disambiguation: compatible, skipped time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: ignore and disambiguation: compatible, skipped time");

// disambiguation: earlier, skipped wall time
zdt = dstStartDay.with(twoThirty, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: ignore and disambiguation: earlier, skipped time");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when option offset: ignore and disambiguation: earlier, skipped time");

// disambiguation: later, skipped wall time
zdt = dstStartDay.with(twoThirty, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore and disambiguation: later, skipped time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when option offset: ignore and disambiguation: later, skipped time");

// disambiguation: compatible, repeated wall time
zdt = dstEndDay.with(oneThirty, {
  offset,
  disambiguation: "compatible"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore and disambiguation: compatible, repeated time");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when option offset: ignore and disambiguation: compatible, repeated time");

// disambiguation: earlier, repeated wall time
zdt = dstEndDay.with(oneThirty, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when option offset: ignore and disambiguation: earlier, repeated time");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when option offset: ignore and disambiguation: earlier, repeated time");

// disambiguation: later, repeated wall time
zdt = dstEndDay.with(oneThirty, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when option offset: ignore and disambiguation: later, repeated time");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when option offset: ignore and disambiguation: later, repeated time");

assert.throws(RangeError, () => dstStartDay.with(twoThirty, {
  offset,
  disambiguation: "reject"
}), "Throws when option offset: ignore and disambiguation: reject");

// Wrong offset.
const bogus = {
  ...twoThirty,
  offset: "+23:59"
};

// offset: ignore, with bogus offset, defers to disambiguation option.
offset = "ignore";
zdt = dstStartDay.with(bogus, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when offset is wrong, option offset:ignore  and disambiguation: earlier");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when offset is wrong, option offset:ignore  and disambiguation: earlier");

zdt = dstStartDay.with(bogus, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset is wrong, option offset:ignore  and disambiguation: later");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset is wrong, option offset:ignore  and disambiguation: later");

// offset: prefer, with bogus offset, defers to disambiguation option.
offset = "prefer";
zdt = dstStartDay.with(bogus, {
  offset,
  disambiguation: "earlier"
});
assert.sameValue(
  zdt.offset,
  "-08:00",
  "Offset result when offset is wrong, option offset: prefer and disambiguation: earlier");
assert.sameValue(
  zdt.hour,
  1,
  "Hour result when offset is wrong, option offset: prefer and disambiguation: earlier");

zdt = dstStartDay.with(bogus, {
  offset,
  disambiguation: "later"
});
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result when offset is wrong, option offset: prefer and disambiguation: later");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result when offset is wrong, option offset: prefer and disambiguation: later");

const doubleTime = new Temporal.ZonedDateTime(972811801_000_000_000n, "America/Vancouver");

// offset: ignore defers to disambiguation option.
offset = "ignore";
assert.sameValue(doubleTime.with({ offset: "-07:00" }, {
  offset,
  disambiguation: "earlier"
}).offset, "-07:00",
  "Offset result when option offset: prefer and disambiguation: later");

assert.sameValue(doubleTime.with({ offset: "-07:00" }, {
  offset,
  disambiguation: "later"
}).offset, "-08:00",
  "Offset result when option offset: prefer and disambiguation: later");
