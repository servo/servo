// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.3.1_b
description: Tests that invalid currency codes are not accepted.
author: Norbert Lindenberg
---*/

var invalidCurrencyCodes = [
    "",
    "€",
    "$",
    "SFr.",
    "DM",
    "KR₩",
    "702",
    "ßP",
    "ınr"
];

invalidCurrencyCodes.forEach(function (code) {
    // this must throw an exception for an invalid currency code
    assert.throws(RangeError, function() {
        var format = new Intl.NumberFormat(["de-de"], {style: "currency", currency: code});
    }, "Invalid currency code '" + code + "' was not rejected.");
});
