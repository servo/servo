// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Different combinations of style options and Temporal.PlainDateTime format correctly.
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

// Use a PlainDateTime with unique values in each field, so as to make it easier
// to test which values appear in the formatted output
const datetime = new Temporal.PlainDateTime(2222, 3, 4, 5, 6, 7, 888, 999, 111);

const expectedDate = {
  // "Monday, March 4, 2222"
  full: {
    year: ["2222", true],
    month: ["3", false],
    day: ["4", true],
  },

  // "March 4, 2222"
  long: {
    year: ["2222", true],
    month: ["3", false],
    day: ["4", true],
  },

  // "Mar 4, 2222"
  medium: {
    year: ["2222", true],
    month: ["3", false],
    day: ["4", true],
  },

  // "3/4/22"
  short: {
    year: ["22", true],
    month: ["3", true],
    day: ["4", true],
  },
};

const expectedTime = {
  // "5:06:07 AM"
  full: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "5:06:07 AM"
  long: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "5:06:07 AM"
  medium: {
    hour: true,
    minute: true,
    second: true,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },

  // "5:06 AM"
  short: {
    hour: true,
    minute: true,
    second: false,
    millisecond: false,
    microsecond: false,
    nanosecond: false,
  },
};

function ensureDateField(result, field, dateStyle) {
  let [searchValue, present] = expectedDate[dateStyle][field];
  let verb = present ? "should" : "should not";

  assert.sameValue(
    result.includes(searchValue),
    present,
    `dateStyle=${dateStyle}: ${field} ${verb} appear`
  );
}

function ensureTimeField(result, field, value, timeStyle) {
  let present = expectedTime[timeStyle][field];
  let verb = present ? "should" : "should not";

  assert.sameValue(
    result.includes(value),
    present,
    `timeStyle=${timeStyle}: ${field} ${verb} appear`
  );
}

for (let dateStyle of dateStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeZone});
  let result = dtf.format(datetime);

  ensureDateField(result, "year", dateStyle);
  ensureDateField(result, "month", dateStyle);
  ensureDateField(result, "day", dateStyle);

  if (dateStyle === "full") {
    assert.sameValue(result.includes("Monday"), true, `dateStyle=${dateStyle}: day of week should appear`);
  }

  assert.sameValue(result.includes("5"), false, `dateStyle=${dateStyle}: hour should not appear`);
  assert.sameValue(result.includes("6"), false, `dateStyle=${dateStyle}: minute should not appear`);
  assert.sameValue(result.includes("7"), false, `dateStyle=${dateStyle}: second should not appear`);
  assert.sameValue(result.includes("888"), false, `dateStyle=${dateStyle}: millisecond should not appear`);
  assert.sameValue(result.includes("999"), false, `dateStyle=${dateStyle}: microsecond should not appear`);
  assert.sameValue(result.includes("111"), false, `dateStyle=${dateStyle}: nanosecond should not appear`);
}

for (let timeStyle of timeStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {timeStyle, timeZone});
  let result = dtf.format(datetime);

  assert.sameValue(result.includes("2"), false, `timeStyle=${timeStyle}: year should not appear`);
  assert.sameValue(result.includes("3"), false, `timeStyle=${timeStyle}: month should not appear`);
  assert.sameValue(result.includes("4"), false, `timeStyle=${timeStyle}: day should not appear`);

  ensureTimeField(result, "hour", "5", timeStyle);
  ensureTimeField(result, "minute", "6", timeStyle);
  ensureTimeField(result, "second", "7", timeStyle);
  ensureTimeField(result, "millisecond", "888", timeStyle);
  ensureTimeField(result, "microsecond", "999", timeStyle);
  ensureTimeField(result, "nanosecond", "111", timeStyle);
}

for (let dateStyle of dateStyles) {
  for (let timeStyle of timeStyles) {
    let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeStyle, timeZone});
    let result = dtf.format(datetime);

    ensureDateField(result, "year", dateStyle);
    ensureDateField(result, "month", dateStyle);
    ensureDateField(result, "day", dateStyle);

    if (dateStyle === "full") {
      assert.sameValue(result.includes("Monday"), true, `dateStyle=${dateStyle}: day of week should appear`);
    }

    ensureTimeField(result, "hour", "5", timeStyle);
    ensureTimeField(result, "minute", "6", timeStyle);
    ensureTimeField(result, "second", "7", timeStyle);
    ensureTimeField(result, "millisecond", "888", timeStyle);
    ensureTimeField(result, "microsecond", "999", timeStyle);
    ensureTimeField(result, "nanosecond", "111", timeStyle);
  }
}
