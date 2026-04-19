// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.3.0_6_1
description: >
    Tests that Date.prototype.toLocaleString & Co. throws the same
    exceptions as Intl.DateTimeFormat.
author: Norbert Lindenberg
---*/

var functions = {
    toLocaleString: Date.prototype.toLocaleString,
    toLocaleDateString: Date.prototype.toLocaleDateString,
    toLocaleTimeString: Date.prototype.toLocaleTimeString
};
var locales = [null, [NaN], ["i"], ["de_DE"]];
var options = [
    {localeMatcher: null},
    {timeZone: "invalid"},
    {hour: "long"},
    {formatMatcher: "invalid"}
];

Object.getOwnPropertyNames(functions).forEach(function (p) {
    var f = functions[p];
    locales.forEach(function (locales) {
        var referenceError, error;
        try {
            var format = new Intl.DateTimeFormat(locales);
        } catch (e) {
            referenceError = e;
        }
        assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.DateTimeFormat for locales " + locales + ".");

        assert.throws(referenceError.constructor, function() {
            var result = f.call(new Date(), locales);
        }, "Date.prototype." + p + " didn't throw exception for locales " + locales + ".");
    });
    
    options.forEach(function (options) {
        var referenceError, error;
        try {
            var format = new Intl.DateTimeFormat([], options);
        } catch (e) {
            referenceError = e;
        }
        assert.notSameValue(referenceError, undefined, "Internal error: Expected exception was not thrown by Intl.DateTimeFormat for options " + JSON.stringify(options) + ".");

        assert.throws(referenceError.constructor, function() {
            var result = f.call(new Date(), [], options);
        }, "Date.prototype." + p + " didn't throw exception for options " + JSON.stringify(options) + ".");
    });
});
