// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: is a String exotic object, let builtinTag be "String".
---*/
assert.sameValue(
  Object.prototype.toString.call(new Date(0)),
  "[object Date]",
  "Object.prototype.toString.call(new Date(0)) returns [object Date]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(new Date(0))),
  "[object Date]",
  "Object.prototype.toString.call(Object(new Date(0))) returns [object Date]"
);
