// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 13.2.1_4_2
description: >
    Tests that Number.prototype.toLocaleString uses the standard
    built-in Intl.NumberFormat constructor.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

taintDataProperty(Intl, "NumberFormat");
(0.0).toLocaleString();
