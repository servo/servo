// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Function declarations are interpreted as lexically-scoped in module code
esid: sec-module-semantics-static-semantics-lexicallydeclarednames
info: |
    ModuleItem : StatementListItem

        1. Return LexicallyDeclaredNames of StatementListItem.

    15.2.1.1 Static Semantics: Early Errors

    - It is a Syntax Error if any element of the LexicallyDeclaredNames of
      ModuleItemList also occurs in the VarDeclaredNames of ModuleItemList.
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

var f;
function f() {}
