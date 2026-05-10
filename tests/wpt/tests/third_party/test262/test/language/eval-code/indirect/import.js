// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The `import` declaration may not appear within eval code
esid: sec-scripts
flags: [module]
info: |
     Eval code is the source text supplied to the built-in eval function. More
     precisely, if the parameter to the built-in eval function is a String, it
     is treated as an ECMAScript Script. The eval code for a particular
     invocation of eval is the global code portion of that Script.

     A.5 Scripts and Modules

     Script:
         ScriptBodyopt

     ScriptBody:
         StatementList
---*/

assert.throws(SyntaxError, function() {
  (0,eval)('import v from "./import.js";');
});
