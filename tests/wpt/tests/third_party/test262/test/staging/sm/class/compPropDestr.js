// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Computed Property Names
info: bugzilla.mozilla.org/show_bug.cgi?id=924688
esid: pending
---*/

var key = "z";
var { [key]: foo } = { z: "bar" };
assert.sameValue(foo, "bar");
