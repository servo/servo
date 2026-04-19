// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 12.2.3_c
description: >
    Tests that Intl.DateTimeFormat provides the required date-time
    format component subsets.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

var locales = ["de-DE", "en-US", "hi-IN", "id-ID", "ja-JP", "th-TH", "zh-Hans-CN", "zh-Hant-TW", "zxx"];
var subsets = [
    {weekday: "long", year: "numeric", month: "numeric", day: "numeric",
        hour: "numeric", minute: "numeric", second: "numeric"},
    {weekday: "long", year: "numeric", month: "numeric", day: "numeric"},
    {year: "numeric", month: "numeric", day: "numeric"},
    {year: "numeric", month: "numeric"},
    {month: "numeric", day: "numeric"},
    {hour: "numeric", minute: "numeric", second: "numeric"},
    {hour: "numeric", minute: "numeric"}
];

locales.forEach(function (locale) {
    subsets.forEach(function (subset) {
        var format = new Intl.DateTimeFormat([locale], subset);
        var actual = format.resolvedOptions();
        getDateTimeComponents().forEach(function (component) {
            if (actual.hasOwnProperty(component)) {
                assert(subset.hasOwnProperty(component),
                        "Unrequested component " + component +
                        " added to requested subset " + JSON.stringify(subset) +
                        "; locale " + locale + ".");
                assert.notSameValue(getDateTimeComponentValues(component).indexOf(actual[component]), -1,
                      "Invalid value " + actual[component] + " for date-time component " + component + "." +
                      " (Testing locale " + locale + "; subset " + JSON.stringify(subset) + ")");
            } else {
                assert.sameValue(subset.hasOwnProperty(component), false,
                        "Missing component " + component +
                        " from requested subset " + JSON.stringify(subset) +
                        "; locale " + locale + ".");
            }
        });
    });
});
