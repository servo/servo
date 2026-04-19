// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1_a
description: >
    Tests that constructing a Collator doesn't create or modify
    unwanted properties on the RegExp constructor.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testForUnwantedRegExpChanges(function () {
    new Intl.Collator("de-DE-u-co-phonebk");
});
