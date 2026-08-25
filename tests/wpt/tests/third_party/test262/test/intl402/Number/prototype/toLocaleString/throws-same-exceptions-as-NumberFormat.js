// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2.1_4_1
description: >
    Tests that Number.prototype.toLocaleString throws the same
    exceptions as Intl.NumberFormat.
author: Norbert Lindenberg
---*/

var locales = [null, [NaN], ["i"], ["de_DE"]];
var options = [
    {localeMatcher: null},
    {style: "invalid"},
    {style: "currency"},
    {style: "currency", currency: "ßP"},
    {maximumSignificantDigits: -Infinity}
];

locales.forEach(function (locales) {
    var referenceError, error;
    try {
        var format = new Intl.NumberFormat(locales);
    } catch (e) {
        referenceError = e;
    }
    assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.NumberFormat for locales " + locales + ".");

    assert.throws(referenceError.constructor, function() {
        var result = (0).toLocaleString(locales);
    }, "Number.prototype.toLocaleString didn't throw exception for locales " + locales + ".");
});

options.forEach(function (options) {
    var referenceError, error;
    try {
        var format = new Intl.NumberFormat([], options);
    } catch (e) {
        referenceError = e;
    }
    assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.NumberFormat for options " + JSON.stringify(options) + ".");

    assert.throws(referenceError.constructor, function() {
        var result = (0).toLocaleString([], options);
    }, "Number.prototype.toLocaleString didn't throw exception for options " + JSON.stringify(options) + ".");
});
