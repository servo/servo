// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Basic tests for Temporal.Instant.toLocaleString
info: |
    Temporal.Instant.prototype.toLocaleString ( [ locales [ , options ] ] )
     ...
    4. Return ? FormatDateTime(dateFormat, instant).
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

const instant = Temporal.Instant.from("1976-11-18T14:23:30Z");

const dtfNY = new Intl.DateTimeFormat("en-US", { timeZone: "America/New_York" });
assert.sameValue(
  instant.toLocaleString("en-US", { timeZone: "America/New_York" }),
  dtfNY.format(instant)
);

const partsNY = dtfNY.formatToParts(instant);
const yearPartNY = findPart(partsNY, "year");
const monthPartNY = findPart(partsNY, "month");
const dayPartNY = findPart(partsNY, "day");
const hourPartNY = findPart(partsNY, "hour");
const minutePartNY = findPart(partsNY, "minute");
const secondPartNY = findPart(partsNY, "second");
const resultNY = instant.toLocaleString("en-US", { timeZone: "America/New_York" });
assert(partsNY.some(part => part.type === "dayPeriod"), "en-US locale has a 12-hour format");
assert(resultNY.includes(yearPartNY), "en-US locale string has a year part");
assert(resultNY.includes(monthPartNY), "en-US locale string has a month part");
assert(resultNY.includes(dayPartNY), "en-US locale string has a day part");
assert(resultNY.includes(hourPartNY), "en-US locale string has an hour part");
assert(resultNY.includes(minutePartNY), "en-US locale string has a minute part");
assert(resultNY.includes(secondPartNY), "en-US locale string has a second part");

const dtfVienna = new Intl.DateTimeFormat("de-AT", { timeZone: "Europe/Vienna" });
assert.sameValue(
  instant.toLocaleString("de-AT", { timeZone: "Europe/Vienna" }),
  dtfVienna.format(instant)
);

const partsVienna = dtfVienna.formatToParts(instant);
const yearPartVienna = findPart(partsVienna, "year");
const monthPartVienna = findPart(partsVienna, "month");
const dayPartVienna = findPart(partsVienna, "day");
const hourPartVienna = findPart(partsVienna, "hour");
const minutePartVienna = findPart(partsVienna, "minute");
const secondPartVienna = findPart(partsVienna, "second");
const resultVienna = instant.toLocaleString("de-AT", { timeZone: "Europe/Vienna" });
assert(!partsVienna.some(part => part.type === "dayPeriod"), "de-AT locale has a 24-hour format");
assert(resultVienna.includes(yearPartVienna), "de-AT locale string has a year part");
assert(resultVienna.includes(monthPartVienna), "de-AT locale string has a month part");
assert(resultVienna.includes(dayPartVienna), "de-AT locale string has a day part");
assert(resultVienna.includes(hourPartVienna), "de-AT locale string has an hour part");
assert(resultVienna.includes(minutePartVienna), "de-AT locale string has a minute part");
assert(resultVienna.includes(secondPartVienna), "de-AT locale string has a second part");
