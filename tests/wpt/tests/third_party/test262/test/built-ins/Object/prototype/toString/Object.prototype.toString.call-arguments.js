// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: has a [[ParameterMap]] internal slot, let builtinTag be "Arguments".
---*/
assert.sameValue(
  Object.prototype.toString.call(function() { return arguments; }()),
  "[object Arguments]",
  "Object.prototype.toString.call(function() { return arguments; }()) returns [object Arguments]"
);
