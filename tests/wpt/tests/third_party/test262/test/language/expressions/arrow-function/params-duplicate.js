// Copyright 2019 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arrow-function-definitions
description: Formal parameters may not contain duplicates
info: |
  # 14.2 Arrow Function Definitions

  When the production

    ArrowParameters:CoverParenthesizedExpressionAndArrowParameterList

  is recognized the following grammar is used to refine the interpretation
  of CoverParenthesizedExpressionAndArrowParameterList:

    ArrowFormalParameters[Yield, Await]:
      (UniqueFormalParameters[?Yield, ?Await])

  # 14.1.2 Static Semantics: Early Errors

  UniqueFormalParameters:FormalParameters

  - It is a Syntax Error if BoundNames of FormalParameters contains any
    duplicate elements.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

0, (a, a) => { };
