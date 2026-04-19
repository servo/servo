// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_7
description: Tests that the option localeMatcher is processed correctly.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testOption(Intl.NumberFormat, "localeMatcher", "string", ["lookup", "best fit"], "best fit", {noReturn: true});
