// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
es5id: 10.4.2-2-c-1
description: Indirect eval code cannot instantiate variable in calling context
---*/

(function() {
  var x = 0;
  (0,eval)("var x = 1;");
  assert.sameValue(x, 0, "x");
}());

assert.sameValue(x, 1);
