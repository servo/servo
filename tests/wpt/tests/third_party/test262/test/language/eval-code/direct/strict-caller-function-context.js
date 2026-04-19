// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: Script will be script if strictCaller is true, inside a function context
info: |
    ...
    10. If strictCaller is true, let strictEval be true.
    ...
    12. Let runningContext be the running execution context.
    ...
---*/

assert.throws(SyntaxError, function() {
  'use strict';
  eval('var public = 1;');
});
