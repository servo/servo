// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 1801690;
var summary = "indexOf function doesn't work correctly with polish letters";

// Prior to this bug being fixed, this would return 0. This is because 'ł'
// truncates to the same 8-bit number as 'B'. We had a guard on the first
// character, but there was a hole in our logic specifically for the
// second character of the needle string.
assert.sameValue("AB".indexOf("Ał"), -1);

