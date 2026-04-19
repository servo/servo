// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Different combinations of style options and Temporal.PlainYearMonth format correctly.
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

// Use a reference day so we can check that it doesn't occur in any string output
const yearmonth = new Temporal.PlainYearMonth(2222, 1, "gregory", 30);

const expected = {
  // "January 2222"
  full: {
    year: ["2222", true],
    month: ["1", false],
    day: ["30", false],
  },

  // "January 2222"
  long: {
    year: ["2222", true],
    month: ["1", false],
    day: ["30", false],
  },

  // "Jan 2222"
  medium: {
    year: ["2222", true],
    month: ["1", false],
    day: ["30", false],
  },

  // "1/22"
  short: {
    year: ["22", true],
    month: ["1", true],
    day: ["30", false],
  },
};

function ensureDateField(result, field, dateStyle) {
  let [searchValue, present] = expected[dateStyle][field];
  let verb = present ? "should" : "should not";

  assert.sameValue(
    result.includes(searchValue),
    present,
    `dateStyle=${dateStyle}: ${field} ${verb} appear`
  );
}

// timeStyle throws when no dateStyle is present.
for (let timeStyle of timeStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {timeStyle, timeZone});
  assert.throws(TypeError, () => dtf.format(yearmonth), `timeStyle=${timeStyle}`);
}

for (let dateStyle of dateStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeZone});
  let result = dtf.format(yearmonth);

  ensureDateField(result, "year", dateStyle);
  ensureDateField(result, "month", dateStyle);
  ensureDateField(result, "day", dateStyle);

  // timeStyle is ignored when dateStyle is present.
  for (let timeStyle of timeStyles) {
    let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeStyle, timeZone});
    assert.sameValue(dtf.format(yearmonth), result, `dateStyle = ${dateStyle}, timeStyle = ${timeStyle}`);
  }
}
