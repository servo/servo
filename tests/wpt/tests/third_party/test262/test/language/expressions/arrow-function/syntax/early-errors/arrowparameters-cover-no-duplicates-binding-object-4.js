// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2.1
description: >
    ArrowParameters[Yield] :
      ...
      CoverParenthesizedExpressionAndArrowParameterList[?Yield]

    CoverParenthesizedExpressionAndArrowParameterList, refined by:

    ArrowFormalParameters[Yield, GeneratorParameter] :
      ( StrictFormalParameters[?Yield, ?GeneratorParameter] )

    ObjectBindingPattern

    BindingPropertyList

    BindingRestElement

    No duplicates

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
var af = ({x}, ...x) => 1;
