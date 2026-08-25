// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks the order of operations on the options argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    5. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
    6. Let numberingSystem be ? GetOption(options, "numberingSystem", "string", undefined, undefined).
    13. Let style be ? GetOption(options, "style", "string", « "long", "short", "narrow", "digital" », "long").
includes: [compareArray.js, temporalHelpers.js]
features: [Intl.DurationFormat]
---*/

var actual = [];

const expected = [
  "get options.localeMatcher",
  "get options.numberingSystem",
  "get options.style",
  "get options.years",
  "get options.yearsDisplay",
  "get options.months",
  "get options.monthsDisplay",
  "get options.weeks",
  "get options.weeksDisplay",
  "get options.days",
  "get options.daysDisplay",
  "get options.hours",
  "get options.hoursDisplay",
  "get options.minutes",
  "get options.minutesDisplay",
  "get options.seconds",
  "get options.secondsDisplay",
  "get options.milliseconds",
  "get options.millisecondsDisplay",
  "get options.microseconds",
  "get options.microsecondsDisplay",
  "get options.nanoseconds",
  "get options.nanosecondsDisplay",
  "get options.fractionalDigits",
];

const options = TemporalHelpers.propertyBagObserver(actual, {}, "options");

let nf = new Intl.DurationFormat(undefined, options);
assert.compareArray(actual, expected);
