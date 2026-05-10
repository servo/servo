// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: If the parse fails, throw a SyntaxError exception (but see also clause 16)
esid: sec-performeval
es5id: 15.1.2.1_A2_T2
description: Checking if execution of "(0,eval)("x = 1; x\u000A++")" fails
negative:
  phase: runtime
  type: SyntaxError
---*/

var x;
(0,eval)("x = 1; x\u000A++");
