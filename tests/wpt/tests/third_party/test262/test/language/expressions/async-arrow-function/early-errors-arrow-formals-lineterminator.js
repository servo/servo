// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AsyncArrowHead
description: async arrows cannot have a line terminator between "async" and the formals
info: |
  14.7 Async Arrow Function Definitions

  AsyncArrowFunction:
    [...]
    CoverCallExpressionAndAsyncArrowHead [no LineTerminator here] => AsyncConciseBody

  Supplemental Syntax

  When processing an instance of the production

  AsyncArrowFunction:
    CoverCallExpressionAndAsyncArrowHead [no LineTerminator here] => AsyncConciseBody

  the interpretation of CoverCallExpressionAndAsyncArrowHead is refined using the following grammar:

  AsyncArrowHead:
    async [no LineTerminator here] ArrowFormalParameters
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

async
(foo) => { }
