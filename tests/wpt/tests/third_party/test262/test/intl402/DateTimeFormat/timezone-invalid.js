// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.4_b
description: Tests that invalid time zone names are not accepted.
author: Norbert Lindenberg
---*/

var invalidTimeZoneNames = [
    "",
    "MEZ", // localized abbreviation
    "Pacific Time", // localized long form
    "cnsha", // BCP 47 time zone code
    "invalid", // as the name says
    "Europe/İstanbul", // non-ASCII letter
    "asıa/baku", // non-ASCII letter
    "europe/brußels"  // non-ASCII letter
];

invalidTimeZoneNames.forEach(function (name) {
    // this must throw an exception for an invalid time zone name
    assert.throws(RangeError, function() {
        var format = new Intl.DateTimeFormat(["de-de"], {timeZone: name});
    }, "Invalid time zone name " + name + " was not rejected.");
});
