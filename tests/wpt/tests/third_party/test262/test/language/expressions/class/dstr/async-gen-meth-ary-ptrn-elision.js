// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision.case
// - src/dstr-binding/default/cls-expr-async-gen-meth.template
/*---
description: Elision advances iterator (class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [generators, async-iteration]
flags: [generated, async]
info: |
    ClassExpression : class BindingIdentifieropt ClassTail

    1. If BindingIdentifieropt is not present, let className be undefined.
    2. Else, let className be StringValue of BindingIdentifier.
    3. Let value be the result of ClassDefinitionEvaluation of ClassTail
       with argument className.
    [...]

    14.5.14 Runtime Semantics: ClassDefinitionEvaluation

    21. For each ClassElement m in order from methods
        a. If IsStatic of m is false, then
           i. Let status be the result of performing
              PropertyDefinitionEvaluation for m with arguments proto and
              false.
        [...]

    Runtime Semantics: PropertyDefinitionEvaluation

    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncGeneratorMethod is strict mode code, let strict be true.
       Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be ! AsyncGeneratorFunctionCreate(Method, UniqueFormalParameters,
       AsyncGeneratorBody, scope, strict).
    [...]


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ Elision ]

    1. Return the result of performing
       IteratorDestructuringAssignmentEvaluation of Elision with iteratorRecord
       as the argument.

    12.14.5.3 Runtime Semantics: IteratorDestructuringAssignmentEvaluation

    Elision : ,

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
       d. If next is false, set iteratorRecord.[[done]] to true.
    2. Return NormalCompletion(empty).

---*/
var first = 0;
var second = 0;
function* g() {
  first += 1;
  yield;
  second += 1;
};


var callCount = 0;
var C = class {
  async *method([,]) {
    assert.sameValue(first, 1);
    assert.sameValue(second, 0);
    callCount = callCount + 1;
  }
};

new C().method(g()).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');    
}).then($DONE, $DONE);
