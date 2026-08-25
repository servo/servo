// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1.1_a
description: >
    Tests that constructing a DateTimeFormat doesn't create or modify
    unwanted properties on the RegExp constructor.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testForUnwantedRegExpChanges(function () {
    new Intl.DateTimeFormat("de-DE-u-ca-gregory");
});

testForUnwantedRegExpChanges(function () {
    new Intl.DateTimeFormat("de-DE-u-ca-gregory", {timeZone: "UTC"});
});
