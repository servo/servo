// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: AnnexB extension not honored in strict mode (IfStatement with a declaration in the second statement position in the global scope)
es6id: B.3.4
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

    B.3.3.2 Changes to GlobalDeclarationInstantiation

    1. 1. Let strict be IsStrict of script
    2. If strict is *false*, then
       [...]
---*/

$DONOTEVALUATE();

if (false) ; else function f() {  }
