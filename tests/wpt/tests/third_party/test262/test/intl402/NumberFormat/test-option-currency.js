// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_17
description: Tests that the option currency is processed correctly.
author: Norbert Lindenberg
---*/

var validValues = ["CNY", "USD", "EUR", "IDR", "jpy", {toString: function () {return "INR";}}];
var invalidValues = ["$", "SFr.", "US$", "ßP", {toString: function () {return;}}];

var defaultLocale = new Intl.NumberFormat().resolvedOptions().locale;

validValues.forEach(function (value) {
    var format, actual, expected;

    // with currency style, we should get the upper case form back
    format = new Intl.NumberFormat([defaultLocale], {style: "currency", currency: value});
    actual = format.resolvedOptions().currency;
    expected = value.toString().toUpperCase();
    assert.sameValue(actual, expected, "Incorrect resolved currency with currency style.");
    
    // without currency style, we shouldn't get any currency back
    format = new Intl.NumberFormat([defaultLocale], {currency: value});
    actual = format.resolvedOptions().currency;
    expected = undefined;
    assert.sameValue(actual, expected, "Incorrect resolved currency with non-currency style.");
    
    // currencies specified through the locale must be ignored
    format = new Intl.NumberFormat([defaultLocale + "-u-cu-krw"], {style: "currency", currency: value});
    actual = format.resolvedOptions().currency;
    expected = value.toString().toUpperCase();
    assert.sameValue(actual, expected, "Incorrect resolved currency with -u-cu- and currency style.");
    
    format = new Intl.NumberFormat([defaultLocale + "-u-cu-krw"], {currency: value});
    actual = format.resolvedOptions().currency;
    expected = undefined;
    assert.sameValue(actual, expected, "Incorrect resolved currency with -u-cu- and non-currency style.");
});

invalidValues.forEach(function (value) {
    assert.throws(RangeError, function () {
            return new Intl.NumberFormat([defaultLocale], {style: "currency", currency: value});
    }, "Invalid currency value " + value + " was not rejected.");
    assert.throws(RangeError, function () {
            return new Intl.NumberFormat([defaultLocale], {currency: value});
    }, "Invalid currency value " + value + " was not rejected.");
    assert.throws(RangeError, function () {
            return new Intl.NumberFormat([defaultLocale + "-u-cu-krw"], {style: "currency", currency: value});
    }, "Invalid currency value " + value + " was not rejected.");
    assert.throws(RangeError, function () {
            return new Intl.NumberFormat([defaultLocale + "-u-cu-krw"], {currency: value});
    }, "Invalid currency value " + value + " was not rejected.");
});
