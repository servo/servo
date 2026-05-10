// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1.1_6_1
description: >
    Tests that String.prototype.localeCompare throws the same
    exceptions as Intl.Collator.
author: Norbert Lindenberg
---*/

var locales = [null, [NaN], ["i"], ["de_DE"]];
var options = [
    {localeMatcher: null},
    {usage: "invalid"},
    {sensitivity: "invalid"}
];

locales.forEach(function (locales) {
    var referenceError, error;
    try {
        var collator = new Intl.Collator(locales);
    } catch (e) {
        referenceError = e;
    }
    assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.Collator for locales " + locales + ".");

    assert.throws(referenceError.constructor, function() {
        var result = "".localeCompare("", locales);
    }, "String.prototype.localeCompare didn't throw exception for locales " + locales + ".");
});

options.forEach(function (options) {
    var referenceError, error;
    try {
        var collator = new Intl.Collator([], options);
    } catch (e) {
        referenceError = e;
    }
    assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.Collator for options " + JSON.stringify(options) + ".");

    assert.throws(referenceError.constructor, function() {
        var result = "".localeCompare("", [], options);
    }, "String.prototype.localeCompare didn't throw exception for options " + JSON.stringify(options) + ".");
});
