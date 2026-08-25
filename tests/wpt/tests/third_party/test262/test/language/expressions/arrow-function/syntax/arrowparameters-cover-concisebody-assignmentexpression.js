// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    ArrowFunction[In, Yield] :
      ArrowParameters[?Yield] [no LineTerminator here] => ConciseBody[?In]

    LineTerminator not present
    ArrowParameters : CoverParenthesizedExpressionAndArrowParameterList[?Yield]
    ConciseBody[In] : [lookahead ≠ { ] AssignmentExpression[?In]
---*/
var af = (x) => x;

assert.sameValue(typeof af, "function");
assert.sameValue(af(1), 1);
