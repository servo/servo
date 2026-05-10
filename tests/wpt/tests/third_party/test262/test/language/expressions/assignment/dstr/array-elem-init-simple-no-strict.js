// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-simple-no-strict.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Identifiers that appear as the DestructuringAssignmentTarget in an AssignmentElement should take on the iterated value corresponding to their position in the ArrayAssignmentPattern. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated, noStrict]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var argument, eval;

var result;
var vals = [];

result = [arguments = 4, eval = 5] = vals;

assert.sameValue(arguments, 4);
assert.sameValue(eval, 5);

assert.sameValue(result, vals);
