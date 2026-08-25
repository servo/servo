// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Different combinations of style options and Temporal.PlainTime format correctly.
locale: [en-US]
features: [Temporal]
---*/

const locale = "en-US";
const timeZone = "Pacific/Apia";

const dateStyles = [
  "full", "long", "medium", "short",
];

const timeStyles = [
  "full", "long", "medium", "short",
];

// Use a PlainTime with unique values in each field, so as to make it easier
// to test which values appear in the formatted output
const time = new Temporal.PlainTime(0, 34, 56, 777, 888, 999);

const expected = {
  // "12:34:56 AM"
  full: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "12:34:56 AM"
  long: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "12:34:56 AM"
  medium: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "12:34 AM"
  short: {
    hour: true,
    minute: true,
    second: false,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },
};

function ensureTimeField(result, field, value, timeStyle) {
  let present = expected[timeStyle][field];
  let verb = present ? "should" : "should not";

  assert.sameValue(
    result.includes(value),
    present,
    `timeStyle=${timeStyle}: ${field} ${verb} appear`
  );
}

// dateStyle throws when no timeStyle is present.
for (let dateStyle of dateStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeZone});
  assert.throws(TypeError, () => dtf.format(time), `dateStyle=${dateStyle}`);
}

for (let timeStyle of timeStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {timeStyle, timeZone});
  let result = dtf.format(time);

  ensureTimeField(result, "hour", "12", timeStyle);
  ensureTimeField(result, "minute", "34", timeStyle);
  ensureTimeField(result, "second", "56", timeStyle);
  ensureTimeField(result, "millisecond", "777", timeStyle);
  ensureTimeField(result, "microsecond", "888", timeStyle);
  ensureTimeField(result, "nanosecond", "999", timeStyle);

  // dateStyle is ignored when timeStyle is present.
  for (let dateStyle of dateStyles) {
    let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeStyle, timeZone});
    assert.sameValue(dtf.format(time), result, `dateStyle = ${dateStyle}, timeStyle = ${timeStyle}`);
  }
}
