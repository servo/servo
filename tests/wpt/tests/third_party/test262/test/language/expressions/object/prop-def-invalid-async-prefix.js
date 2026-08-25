// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  async is not a valid prefix of an identifier reference
esid: sec-object-initializer
info: |
    PropertyDefinition:
      IdentifierReference
      CoverInitializedName
      PropertyName : AssignmentExpression
      MethodDefinition

    MethodDefinition:
      PropertyName ( UniqueFormalParameters ) { FunctionBody }
      AsyncMethod

    AsyncMethod:
      async [no LineTerminator here] PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }
      VariableDeclaration : BindingPattern Initializer

      1. Let rhs be the result of evaluating Initializer.
      2. Let rval be GetValue(rhs).
      3. ReturnIfAbrupt(rval).
      4. Return the result of performing BindingInitialization for
         BindingPattern passing rval and undefined as arguments.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

({async async});
