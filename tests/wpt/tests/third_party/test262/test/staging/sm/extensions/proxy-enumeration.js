// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var list = Object.getOwnPropertyNames(this);
var found = list.indexOf("Proxy") != -1;
assert.sameValue(found, true)
