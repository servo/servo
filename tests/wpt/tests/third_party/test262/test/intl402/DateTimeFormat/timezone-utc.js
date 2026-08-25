// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.4_a
description: Tests that valid time zone names are accepted.
author: Norbert Lindenberg
---*/

var validTimeZoneNames = [
    "UTC",
    "utc" // time zone names are case-insensitive
];

validTimeZoneNames.forEach(function (name) {
    // this must not throw an exception for a valid time zone name
    var format = new Intl.DateTimeFormat(["de-de"], {timeZone: name});
    assert.sameValue(format.resolvedOptions().timeZone, name.toUpperCase(), "Time zone name " + name + " was not correctly accepted.");
});
