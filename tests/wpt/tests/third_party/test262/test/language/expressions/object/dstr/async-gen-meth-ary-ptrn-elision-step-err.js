// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-step-err.case
// - src/dstr-binding/error/async-gen-meth.template
/*---
description: Elision advances iterator and forwards abrupt completions (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [generators, async-iteration]
flags: [generated]
info: |
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

---*/
var following = 0;
var iter =function* () {
  throw new Test262Error();
  following += 1;
}();


var obj = {
  async *method([,]) {
    
  }
};

assert.throws(Test262Error, function() {
  obj.method(iter);
});

iter.next();
assert.sameValue(following, 0, 'Iterator was properly closed.');
