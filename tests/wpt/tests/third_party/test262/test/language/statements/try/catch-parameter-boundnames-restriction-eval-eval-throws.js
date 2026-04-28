// Copyright (c) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-strict-mode-of-ecmascript
description: >
    It is a SyntaxError if a CatchParameter occurs within strict mode code and BoundNames of CatchParameter contains either eval or arguments (13.15.1).
flags: [onlyStrict]
---*/

assert.throws(SyntaxError, function() {
  eval("try {} catch (eval) { }");
});
