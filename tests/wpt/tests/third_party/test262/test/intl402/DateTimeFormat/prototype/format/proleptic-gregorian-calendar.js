// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.3.2_FDT_7_a_iv
description: >
    Tests that format uses a proleptic Gregorian calendar with no year
    0.
author: Norbert Lindenberg
---*/

var dates = [
    0, // January 1, 1970
    -62151602400000, // in June 1 BC
    -8640000000000000 // beginning of ECMAScript time
];

var format = new Intl.DateTimeFormat(["en-US"], {year: "numeric", era: "short", timeZone: "UTC"});

// this test requires a Gregorian calendar, which we usually find in the US
assert.sameValue(format.resolvedOptions().calendar, "gregory", "Internal error: Didn't find Gregorian calendar");

dates.forEach(function (date) {
    var year = new Date(date).getUTCFullYear();
    var expectedYear = year <= 0 ? 1 - year : year;
    var expectedYearString = expectedYear.toLocaleString(["en-US"], {useGrouping: false});
    var expectedEra = year <= 0 ? /BC/ : /AD|(?:^|[^B])CE/;
    var dateString = format.format(date);
    assert.notSameValue(dateString.indexOf(expectedYearString), -1, "Formatted year doesn't contain expected year – expected " + expectedYearString + ", got " + dateString + ".");
    assert(expectedEra.test(dateString), "Formatted year doesn't contain expected era – expected " + expectedEra + ", got " + dateString + ".");
});
