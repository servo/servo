// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 10.2.3_b
description: >
    Tests that Intl.Collator does not accept Unicode locale  extension
    keys and values that are not allowed.
author: Norbert Lindenberg
includes: [compareArray.js]
---*/

var testArray = [
        "hello", "你好", "こんにちは",
        "pêche", "peché", "1", "9", "10",
        "ụ\u031B", "u\u031B\u0323", "ư\u0323", "u\u0323\u031B",
        "Å", "Å", "A\u030A"
];

var defaultCollator = new Intl.Collator();
var defaultOptions = defaultCollator.resolvedOptions();
var defaultOptionsJSON = JSON.stringify(defaultOptions);
var defaultLocale = defaultOptions.locale;
var defaultSortedArray = testArray.slice(0).sort(defaultCollator.compare);

var keyValues = {
    "co": ["standard", "search", "invalid"],
    "ka": ["noignore", "shifted", "invalid"],
    "kb": ["true", "false", "invalid"],
    "kc": ["true", "false", "invalid"],
    "kh": ["true", "false", "invalid"],
    "kk": ["true", "false", "invalid"],
    "kr": ["latn-hira-hani", "hani-hira-latn", "invalid"],
    "ks": ["level1", "level2", "level3", "level4", "identic", "invalid"],
    "vt": ["1234-5678-9abc-edf0", "invalid"]
};

Object.getOwnPropertyNames(keyValues).forEach(function (key) {
    keyValues[key].forEach(function (value) {
        var collator = new Intl.Collator([defaultLocale + "-u-" + key + "-" + value]);
        var options = collator.resolvedOptions();
        assert.sameValue(options.locale, defaultLocale, "Locale " + options.locale + " is affected by key " + key + "; value " + value + ".");
        assert.sameValue(JSON.stringify(options), defaultOptionsJSON, "Resolved options " + JSON.stringify(options) + " are affected by key " + key + "; value " + value + ".");
        assert.compareArray(testArray.sort(collator.compare), defaultSortedArray);
    });
});
