// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-async-generator-function-definitions-static-semantics-early-errors
description: >
  It is a SyntaxError if BoundNames of FormalParameters also occurs in the
  LexicallyDeclaredNames of AsyncFunctionBody
info: |
  It is a Syntax Error if any element of the BoundNames of FormalParameters
  also occurs in the LexicallyDeclaredNames of GeneratorBody.
negative:
  phase: parse
  type: SyntaxError
features: [async-iteration]
---*/

$DONOTEVALUATE();

(async function*(a) { const a = 0; });
