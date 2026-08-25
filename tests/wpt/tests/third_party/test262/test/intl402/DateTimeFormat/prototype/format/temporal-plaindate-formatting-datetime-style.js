// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Different combinations of style options and Temporal.PlainDate format correctly.
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

const date = new Temporal.PlainDate(2021, 8, 4);

const expected = {
  // "Wednesday, August 4, 2021"
  full: {
    year: ["2021", true],
    month: ["8", false],
    day: ["4", true],
  },

  // "August 4, 2021"
  long: {
    year: ["2021", true],
    month: ["8", false],
    day: ["4", true],
  },

  // "Aug 4, 2021"
  medium: {
    year: ["2021", true],
    month: ["8", false],
    day: ["4", true],
  },

  // "8/4/21"
  short: {
    year: ["21", true],
    month: ["8", true],
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
  assert.throws(TypeError, () => dtf.format(date), `timeStyle=${timeStyle}`);
}

for (let dateStyle of dateStyles) {
  let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeZone});
  let result = dtf.format(date);

  ensureDateField(result, "year", dateStyle);
  ensureDateField(result, "month", dateStyle);
  ensureDateField(result, "day", dateStyle);

  // timeStyle is ignored when dateStyle is present.
  for (let timeStyle of timeStyles) {
    let dtf = new Intl.DateTimeFormat(locale, {dateStyle, timeStyle, timeZone});
    assert.sameValue(dtf.format(date), result, `dateStyle = ${dateStyle}, timeStyle = ${timeStyle}`);
  }
}
