// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: AnnexB extension not honored in strict mode (IfStatement with a declaration in the first statement position in the global scope)
esid: sec-if-statement
es6id: 13.6
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]

    The above rules are only applied when parsing code that is not strict mode code.
---*/

$DONOTEVALUATE();

if (true) function f() {} else ;
