// Copyright 2020 Qu Xing. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: For statement
info: |
    ...
    9. Perform the following substeps in an implementation-dependent order, possibly interleaving parsing and error detection:
      a. Let script be the ECMAScript code that is the result of parsing ! UTF16DecodeString(x),for the goal symbol Script. 
         If the parse fails, throw a SyntaxError exception. If any early errors are detected, throw a SyntaxError exception 
         (but see also clause 16).
    ...
---*/

assert.throws(SyntaxError, function() {
  (0, eval)("for(;false;)");
});
