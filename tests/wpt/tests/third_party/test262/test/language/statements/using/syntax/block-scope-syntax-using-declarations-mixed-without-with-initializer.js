// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-static-semantics-early-errors
description: >
    using declarations mixed: without, with initializer
info: |
    Static Semantics : Early Errors
      LexicalBinding : BindingIdentifier Initializer?

      - It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing
        this LexicalBinding is true.

    Static Semantics : IsConstantDeclaration
      UsingDeclaration :
        `using` BindingList ;

      1. Return true.

negative:
  phase: parse
  type: SyntaxError
features: [explicit-resource-management]
---*/

$DONOTEVALUATE();
{
  using x, y = null;
}
