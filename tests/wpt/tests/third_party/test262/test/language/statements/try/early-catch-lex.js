// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-try-statement-static-semantics-early-errors
es6id: 13.15.1
description: >
    It is a Syntax Error if any element of the BoundNames of CatchParameter
    also occurs in the LexicallyDeclaredNames of Block.
negative:
  phase: parse
  type: SyntaxError
features: [let]
---*/

$DONOTEVALUATE();

try { } catch (x) { let x; }
