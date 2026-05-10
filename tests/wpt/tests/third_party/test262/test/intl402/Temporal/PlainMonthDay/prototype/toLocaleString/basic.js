// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: Basic tests for Temporal.PlainMonthDay.toLocaleString
info: |
    Temporal.PlainMonthDay.prototype.toLocaleString ( [ locales [ , options ] ] )
     ...
    4. Return ? FormatDateTime(dateFormat, plainMonthDay).
    ...

    FormatDateTime ( dateTimeFormat, x )
    1. Let parts be ? PartitionDateTimePattern(dateTimeFormat, x).
    2. Let result be the empty String.
    3. For each Record { [[Type]], [[Value]] } part of parts, do
        a. Set result to the string-concatenation of result and part.[[Value]].
    4. Return result.

    FormatDateTimeToParts ( dateTimeFormat, x )
    1. Let parts be ? PartitionDateTimePattern(dateTimeFormat, x).
    2. Let result be ! ArrayCreate(0).
    3. Let n be 0.
    4. For each Record { [[Type]], [[Value]] } part of parts, do
           a. Let O be OrdinaryObjectCreate(%Object.prototype%).
           b. Perform ! CreateDataPropertyOrThrow(O, "type", part.[[Type]]).
           c. Perform ! CreateDataPropertyOrThrow(O, "value", part.[[Value]]).
           d. Perform ! CreateDataPropertyOrThrow(result, ! ToString(𝔽(n)), O).
           e. Increment n by 1.
    5. Return result.
locale: [en-US, de-AT]
---*/

function findPart(parts, expectedType) {
  return parts.find(({ type }) => type === expectedType).value;
}

const calendar = new Intl.DateTimeFormat("en-US").resolvedOptions().calendar;
const monthday = Temporal.PlainMonthDay.from({
  monthCode: "M11",
  day: 18,
  calendar
});

const dtfNY = new Intl.DateTimeFormat("en-US", { timeZone: "America/New_York" });
assert.sameValue(
  monthday.toLocaleString("en-US", { timeZone: "America/New_York" }),
  dtfNY.format(monthday)
);

const partsNY = dtfNY.formatToParts(monthday);
const monthPartNY = findPart(partsNY, "month");
const dayPartNY = findPart(partsNY, "day");
const resultNY = monthday.toLocaleString("en-US", { timeZone: "America/New_York" });
assert(resultNY.includes(monthPartNY), "en-US locale string has a month part");
assert(resultNY.includes(dayPartNY), "en-US locale string has a day part");

const dtfVienna = new Intl.DateTimeFormat("de-AT", { timeZone: "Europe/Vienna", calendar });
assert.sameValue(
  monthday.toLocaleString("de-AT", { timeZone: "Europe/Vienna", calendar }),
  dtfVienna.format(monthday)
);

const partsVienna = dtfVienna.formatToParts(monthday);
const monthPartVienna = findPart(partsVienna, "month");
const dayPartVienna = findPart(partsVienna, "day");
const resultVienna = monthday.toLocaleString("de-AT", { timeZone: "Europe/Vienna", calendar });
assert(resultVienna.includes(monthPartVienna), "de-AT locale string has a month part");
assert(resultVienna.includes(dayPartVienna), "de-AT locale string has a day part");
