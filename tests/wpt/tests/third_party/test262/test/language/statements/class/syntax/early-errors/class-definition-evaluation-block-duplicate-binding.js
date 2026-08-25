// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1.1
description: >
    Block : { StatementList }

    It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
{
  class A {}
  class A {}
}
