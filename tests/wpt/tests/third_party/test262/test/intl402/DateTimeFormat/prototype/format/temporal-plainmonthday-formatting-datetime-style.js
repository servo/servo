// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Different combinations of style options and Temporal.PlainMonthDay format correctly.
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

// Use a reference year so we can check that it doesn't occur in any string output
const monthday = new Temporal.PlainMonthDay(3, 4, "gregory", 5678);

const expected = {
  // "March 4"
  full: {
    year: ["5678", false],
    month: ["3", false],
    day: ["4", true],
  },

  // "March 4"
  long: {
    year: ["5678", false],
    month: ["3", false],
    day: ["4", true],
  },

  // "Mar 4"
  medium: {
    year: ["5678", false],
    month: ["3", false],
    day: ["4", true],
  },

  // "3/4"
  short: {
    year: ["78", false],
    month: ["3", true],
    day: ["4", true],
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
  assert.throws(TypeError, () => dtf.format(monthday), `timeStyle=${timeStyle}`);
}

for (let dateStyle of dateStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeZone});
  let result = dtf.format(monthday);

  ensureDateField(result, "year", dateStyle);
  ensureDateField(result, "month", dateStyle);
  ensureDateField(result, "day", dateStyle);

  // timeStyle is ignored when dateStyle is present.
  for (let timeStyle of timeStyles) {
    let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeStyle, timeZone});
    assert.sameValue(dtf.format(monthday), result, `dateStyle = ${dateStyle}, timeStyle = ${timeStyle}`);
  }
}
