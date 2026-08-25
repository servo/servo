// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-identifier-yield-ident-invalid.case
// - src/dstr-assignment/syntax/assignment-expr.template
/*---
description: yield is not a valid IdentifierReference in an AssignmentProperty within strict mode code. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
$DONOTEVALUATE();

0, { yield } = {};
