// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-not-final-obj.case
// - src/dstr-binding/default/async-gen-func-expr.template
/*---
description: Rest element (object binding pattern) may not be followed by any element (async generator function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * ( FormalParameters ) {
        AsyncGeneratorBody }

        [...]
        3. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, scope, strict).
        [...]


    13.3.3 Destructuring Binding Patterns
    ArrayBindingPattern[Yield] :
        [ Elisionopt BindingRestElement[?Yield]opt ]
        [ BindingElementList[?Yield] ]
        [ BindingElementList[?Yield] , Elisionopt BindingRestElement[?Yield]opt ]
---*/
$DONOTEVALUATE();


var callCount = 0;
var f;
f = async function*([...{ x }, y]) {
  
  callCount = callCount + 1;
};

f([1, 2, 3]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
