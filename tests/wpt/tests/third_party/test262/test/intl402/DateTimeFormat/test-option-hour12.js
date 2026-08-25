// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1.1_18
description: Tests that the option hour12 is processed correctly.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testOption(Intl.DateTimeFormat, "hour12", "boolean", undefined, undefined,
    {extra: {any: {hour: "numeric", minute: "numeric"}}});
testOption(Intl.DateTimeFormat, "hour12", "boolean", undefined, undefined,
    {noReturn: true});
