// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1_13
description: >
    Tests that the options numeric and caseFirst are processed
    correctly.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testOption(Intl.Collator, "numeric", "boolean", undefined, undefined, {isOptional: true});
testOption(Intl.Collator, "caseFirst", "string", ["upper", "lower", "false"], undefined, {isOptional: true});
