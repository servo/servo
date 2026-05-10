// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  pending
esid: pending
---*/

testJSONSyntaxError('{"Numbers cannot have leading zeroes": 013}');
