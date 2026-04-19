// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 12.3.3
description: >
    Tests that the object returned by
    Intl.DateTimeFormat.prototype.resolvedOptions  has the right
    properties.
author: Norbert Lindenberg
includes: [testIntl.js, propertyHelper.js]
---*/

var actual = new Intl.DateTimeFormat().resolvedOptions();

var actual2 = new Intl.DateTimeFormat().resolvedOptions();
assert.notSameValue(actual2, actual, "resolvedOptions returned the same object twice.");

var calendars = allCalendars();

// this assumes the default values where the specification provides them
assert(isCanonicalizedStructurallyValidLanguageTag(actual.locale),
       "Invalid locale: " + actual.locale);
assert.notSameValue(calendars.indexOf(actual.calendar), -1,
                    "Invalid calendar: " + actual.calendar);
assert(isValidNumberingSystem(actual.numberingSystem),
       "Invalid numbering system: " + actual.numberingSystem);
assert(isCanonicalizedStructurallyValidTimeZoneName(actual.timeZone),
       "Invalid time zone: " + actual.timeZone);
assert.notSameValue(["2-digit", "numeric"].indexOf(actual.year), -1,
                    "Invalid year: " + actual.year);
assert.notSameValue(["2-digit", "numeric", "narrow", "short", "long"].indexOf(actual.month), -1,
                    "Invalid month: " + actual.month);
assert.notSameValue(["2-digit", "numeric"].indexOf(actual.day), -1,
                    "Invalid day: " + actual.day);

var dataPropertyDesc = { writable: true, enumerable: true, configurable: true };
verifyProperty(actual, "locale", dataPropertyDesc);
verifyProperty(actual, "calendar", dataPropertyDesc);
verifyProperty(actual, "numberingSystem", dataPropertyDesc);
verifyProperty(actual, "timeZone", dataPropertyDesc);
verifyProperty(actual, "weekday", undefined);
verifyProperty(actual, "era", undefined);
verifyProperty(actual, "year", dataPropertyDesc);
verifyProperty(actual, "month", dataPropertyDesc);
verifyProperty(actual, "day", dataPropertyDesc);
verifyProperty(actual, "hour", undefined);
verifyProperty(actual, "minute", undefined);
verifyProperty(actual, "second", undefined);
verifyProperty(actual, "timeZoneName", undefined);
verifyProperty(actual, "hour12", undefined);
