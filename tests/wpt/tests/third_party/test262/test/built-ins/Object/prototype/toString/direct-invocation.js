// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: Else, let builtinTag be "Object".
---*/
assert.sameValue(
  Object.prototype.toString(),
  "[object Object]",
  "Object.prototype.toString() returns [object Object]"
);
assert.sameValue(
  {}.toString(),
  "[object Object]",
  "({}).toString() returns [object Object]"
);
