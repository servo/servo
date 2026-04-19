// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: Basic tests for Temporal.PlainDate.toLocaleString
info: |
    Temporal.PlainDate.prototype.toLocaleString ( [ locales [ , options ] ] )
     ...
    4. Return ? FormatDateTime(dateFormat, plainDate).
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
features: [Temporal]
locale: [en-US, de-AT]
---*/

function findPart(parts, expectedType) {
  return parts.find(({ type }) => type === expectedType).value;
}

const date = Temporal.PlainDate.from("1976-11-18T15:23:30");

const dtfNY = new Intl.DateTimeFormat("en-US", { timeZone: "America/New_York" });
assert.sameValue(
  date.toLocaleString("en-US", { timeZone: "America/New_York" }),
  dtfNY.format(date)
);

const partsNY = dtfNY.formatToParts(date);
const yearPartNY = findPart(partsNY, "year");
const monthPartNY = findPart(partsNY, "month");
const dayPartNY = findPart(partsNY, "day");
const resultNY = date.toLocaleString("en-US", { timeZone: "America/New_York" });
assert(resultNY.includes(yearPartNY), "en-US locale string has a year part");
assert(resultNY.includes(monthPartNY), "en-US locale string has a month part");
assert(resultNY.includes(dayPartNY), "en-US locale string has a day part");

const dtfVienna = new Intl.DateTimeFormat("de-AT", { timeZone: "Europe/Vienna" });
assert.sameValue(
  date.toLocaleString("de-AT", { timeZone: "Europe/Vienna" }),
  dtfVienna.format(date)
);

const partsVienna = dtfVienna.formatToParts(date);
const yearPartVienna = findPart(partsVienna, "year");
const monthPartVienna = findPart(partsVienna, "month");
const dayPartVienna = findPart(partsVienna, "day");
const resultVienna = date.toLocaleString("de-AT", { timeZone: "Europe/Vienna" });
assert(resultVienna.includes(yearPartVienna), "de-AT locale string has a year part");
assert(resultVienna.includes(monthPartVienna), "de-AT locale string has a month part");
assert(resultVienna.includes(dayPartVienna), "de-AT locale string has a day part");
