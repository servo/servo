// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-put-unresolvable-no-strict.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Outside of strict mode, if the the assignment target is an unresolvable reference, a new `var` binding should be created in the environment record. (AssignmentExpression)
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
{

var result;
var vals = [];

result = [ unresolvable ] = vals;



assert.sameValue(result, vals);

}

assert.sameValue(unresolvable, undefined);
