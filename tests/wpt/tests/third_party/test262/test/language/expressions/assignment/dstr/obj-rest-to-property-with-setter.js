// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-to-property-with-setter.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When DestructuringAssignmentTarget is an object property setter, its value should be binded as rest object. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [object-rest, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var settedValue;
var executedGetter = false;
var src = {
    get y() { executedGetter = true; },
    set y(v) {
        settedValue = v;
    }
}
src.y = undefined;

var result;
var vals = { x: 1, y: 2};

result = {...src.y} = vals;

assert.sameValue(settedValue.x, 1);
assert.sameValue(settedValue.y, 2);
assert(!executedGetter, "The property should not be accessed");


assert.sameValue(result, vals);
