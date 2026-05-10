// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Basic tests for Temporal.PlainTime.toLocaleString
info: |
    Temporal.PlainTime.prototype.toLocaleString ( [ locales [ , options ] ] )
     ...
    4. Return ? FormatDateTime(dateFormat, plainTime).
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

const time = Temporal.PlainTime.from("1976-11-18T15:23:30");

const dtfNY = new Intl.DateTimeFormat("en-US", { timeZone: "America/New_York" });
assert.sameValue(
  time.toLocaleString("en-US", { timeZone: "America/New_York" }),
  dtfNY.format(time)
);

const partsNY = dtfNY.formatToParts(time);
const hourPartNY = findPart(partsNY, "hour");
const minutePartNY = findPart(partsNY, "minute");
const secondPartNY = findPart(partsNY, "second");
const resultNY = time.toLocaleString("en-US", { timeZone: "America/New_York" });
assert(partsNY.some(part => part.type === "dayPeriod"), "en-US locale has a 12-hour format");
assert(resultNY.includes(hourPartNY), "en-US locale string has an hour part");
assert(resultNY.includes(minutePartNY), "en-US locale string has a minute part");
assert(resultNY.includes(secondPartNY), "en-US locale string has a second part");

const dtfVienna = new Intl.DateTimeFormat("de-AT", { timeZone: "Europe/Vienna" });
assert.sameValue(
  time.toLocaleString("de-AT", { timeZone: "Europe/Vienna" }),
  dtfVienna.format(time)
);

const partsVienna = dtfVienna.formatToParts(time);
const hourPartVienna = findPart(partsVienna, "hour");
const minutePartVienna = findPart(partsVienna, "minute");
const secondPartVienna = findPart(partsVienna, "second");
const resultVienna = time.toLocaleString("de-AT", { timeZone: "Europe/Vienna" });
assert(!partsVienna.some(part => part.type === "dayPeriod"), "de-AT locale has a 24-hour format");
assert(resultVienna.includes(hourPartVienna), "de-AT locale string has an hour part");
assert(resultVienna.includes(minutePartVienna), "de-AT locale string has a minute part");
assert(resultVienna.includes(secondPartVienna), "de-AT locale string has a second part");
