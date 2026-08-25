// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: If the this value is null, return "[object Null]".
---*/
assert.sameValue(
  Object.prototype.toString.call(null),
  "[object Null]",
  "Object.prototype.toString.call(null) returns [object Null]"
);
