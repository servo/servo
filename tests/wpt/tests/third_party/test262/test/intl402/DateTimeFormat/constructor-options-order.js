// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks the order of getting options for the DateTimeFormat constructor.
includes: [compareArray.js, temporalHelpers.js]
features:
  - Intl.DateTimeFormat-dayPeriod
  - Intl.DateTimeFormat-fractionalSecondDigits
  - Intl.DateTimeFormat-datetimestyle
locale: [en]
---*/

const expected = [
  // CreateDateTimeFormat step 4.
  "get options.localeMatcher",
  "get options.localeMatcher.toString",
  "call options.localeMatcher.toString",
  "get options.calendar",
  "get options.calendar.toString",
  "call options.calendar.toString",
  "get options.numberingSystem",
  "get options.numberingSystem.toString",
  "call options.numberingSystem.toString",
  // CreateDateTimeFormat step 12.
  "get options.hour12",
  // CreateDateTimeFormat step 13.
  "get options.hourCycle",
  "get options.hourCycle.toString",
  "call options.hourCycle.toString",
  // CreateDateTimeFormat step 29.
  "get options.timeZone",
  "get options.timeZone.toString",
  "call options.timeZone.toString",
  // CreateDateTimeFormat step 36.
  "get options.weekday",
  "get options.weekday.toString",
  "call options.weekday.toString",
  "get options.era",
  "get options.era.toString",
  "call options.era.toString",
  "get options.year",
  "get options.year.toString",
  "call options.year.toString",
  "get options.month",
  "get options.month.toString",
  "call options.month.toString",
  "get options.day",
  "get options.day.toString",
  "call options.day.toString",
  "get options.dayPeriod",
  "get options.dayPeriod.toString",
  "call options.dayPeriod.toString",
  "get options.hour",
  "get options.hour.toString",
  "call options.hour.toString",
  "get options.minute",
  "get options.minute.toString",
  "call options.minute.toString",
  "get options.second",
  "get options.second.toString",
  "call options.second.toString",
  "get options.fractionalSecondDigits",
  "get options.fractionalSecondDigits.valueOf",
  "call options.fractionalSecondDigits.valueOf",
  "get options.timeZoneName",
  "get options.timeZoneName.toString",
  "call options.timeZoneName.toString",
  // CreateDateTimeFormat step 37.
  "get options.formatMatcher",
  "get options.formatMatcher.toString",
  "call options.formatMatcher.toString",
  // CreateDateTimeFormat step 38.
  "get options.dateStyle",
  // CreateDateTimeFormat step 40.
  "get options.timeStyle",
];

const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  calendar: "gregory",
  dateStyle: undefined,
  day: "numeric",
  dayPeriod: "long",
  era: "long",
  formatMatcher: "best fit",
  fractionalSecondDigits: 3,
  hour: "numeric",
  hour12: true,
  hourCycle: "h24",
  localeMatcher: "best fit",
  minute: "numeric",
  month: "numeric",
  numberingSystem: "latn",
  second: "numeric",
  timeStyle: undefined,
  timeZone: "UTC",
  timeZoneName: "long",
  weekday: "long",
  year: "numeric",
}, "options");

new Intl.DateTimeFormat("en", options);

assert.compareArray(actual, expected);
