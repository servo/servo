// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks the order of getting options for the DateTimeFormat constructor.
includes: [compareArray.js]
features: [Intl.DateTimeFormat-datetimestyle]
---*/

// To be merged into constructor-options-order.js when the feature is removed.

const expected = [
  // CreateDateTimeFormat step 4.
  "localeMatcher",
  // CreateDateTimeFormat step 12.
  "hour12",
  // CreateDateTimeFormat step 13.
  "hourCycle",
  // CreateDateTimeFormat step 29.
  "timeZone",
  // CreateDateTimeFormat step 36.
  "weekday",
  "era",
  "year",
  "month",
  "day",
  "hour",
  "minute",
  "second",
  "timeZoneName",
  "formatMatcher",
  // CreateDateTimeFormat step 38.
  "dateStyle",
  // CreateDateTimeFormat step 40.
  "timeStyle",
];

const actual = [];

const options = {
  get dateStyle() {
    actual.push("dateStyle");
    return undefined;
  },

  get day() {
    actual.push("day");
    return "numeric";
  },

  get era() {
    actual.push("era");
    return "long";
  },

  get formatMatcher() {
    actual.push("formatMatcher");
    return "best fit";
  },

  get hour() {
    actual.push("hour");
    return "numeric";
  },

  get hour12() {
    actual.push("hour12");
    return true;
  },

  get hourCycle() {
    actual.push("hourCycle");
    return "h24";
  },

  get localeMatcher() {
    actual.push("localeMatcher");
    return "best fit";
  },

  get minute() {
    actual.push("minute");
    return "numeric";
  },

  get month() {
    actual.push("month");
    return "numeric";
  },

  get second() {
    actual.push("second");
    return "numeric";
  },

  get timeStyle() {
    actual.push("timeStyle");
    return undefined;
  },

  get timeZone() {
    actual.push("timeZone");
    return "UTC";
  },

  get timeZoneName() {
    actual.push("timeZoneName");
    return "long";
  },

  get weekday() {
    actual.push("weekday");
    return "long";
  },

  get year() {
    actual.push("year");
    return "numeric";
  },
};

new Intl.DateTimeFormat("en", options);

assert.compareArray(actual, expected);
