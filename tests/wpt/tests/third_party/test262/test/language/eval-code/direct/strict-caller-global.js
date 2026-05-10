// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: Script will be script if strictCaller is true
info: |
    ...
    10. If strictCaller is true, let strictEval be true.
    ...
    12. Let runningContext be the running execution context.
    ...
negative:
  phase: runtime
  type: SyntaxError
flags: [onlyStrict]
---*/

// Although the `try` statement is a more precise mechanism for detecting
// runtime errors, the behavior under test is only observable for a direct eval
// call when the call is made from the global scope. This forces the use of
// the more coarse-grained `negative` frontmatter to assert the expected error.

eval('var public = 1;');
