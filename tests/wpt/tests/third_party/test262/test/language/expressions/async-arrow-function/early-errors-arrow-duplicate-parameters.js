// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-arrow-function-definitions
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
features: [async-functions]
---*/

$DONOTEVALUATE();

async(a, a) => { }
