// Copyright (C) 2019 Tiancheng "Timothy" Gu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  * is not a valid prefix of an identifier reference
esid: sec-object-initializer
info: |
    PropertyDefinition:
      IdentifierReference
      CoverInitializedName
      PropertyName : AssignmentExpression
      MethodDefinition
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

({* foo});
