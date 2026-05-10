// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 13.1.1_6_2
description: >
    Tests that String.prototype.localeCompare uses the standard
    built-in Intl.Collator constructor.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

taintDataProperty(Intl, "Collator");
"a".localeCompare("b");
