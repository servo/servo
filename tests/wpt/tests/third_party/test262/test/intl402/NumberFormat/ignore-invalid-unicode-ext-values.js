// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 11.2.3_b
description: >
    Tests that Intl.NumberFormat does not accept Unicode locale
    extension keys and values that are not allowed.
author: Norbert Lindenberg
---*/

var locales = ["ja-JP", "zh-Hans-CN", "zh-Hant-TW"];
var input = 1234567.89;

locales.forEach(function (locale) {
    var defaultNumberFormat = new Intl.NumberFormat([locale]);
    var defaultOptions = defaultNumberFormat.resolvedOptions();
    var defaultOptionsJSON = JSON.stringify(defaultOptions);
    var defaultLocale = defaultOptions.locale;
    var defaultFormatted = defaultNumberFormat.format(input);

    var keyValues = {
        "cu": ["USD", "EUR", "JPY", "CNY", "TWD", "invalid"],
        "nu": ["native", "traditio", "finance", "invalid"]
    };
    
    Object.getOwnPropertyNames(keyValues).forEach(function (key) {
        keyValues[key].forEach(function (value) {
            var numberFormat = new Intl.NumberFormat([locale + "-u-" + key + "-" + value]);
            var options = numberFormat.resolvedOptions();
            assert.sameValue(options.locale, defaultLocale, "Locale " + options.locale + " is affected by key " + key + "; value " + value + ".");
            assert.sameValue(JSON.stringify(options), defaultOptionsJSON, "Resolved options " + JSON.stringify(options) + " are affected by key " + key + "; value " + value + ".");
            assert.sameValue(numberFormat.format(input), defaultFormatted, "Formatted value " + numberFormat.format(input) + " is affected by key " + key + "; value " + value + ".");
        });
    });
});
