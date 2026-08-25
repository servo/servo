// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-step-err.case
// - src/dstr-binding/error/async-gen-func-decl.template
/*---
description: Elision advances iterator and forwards abrupt completions (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [generators, async-iteration]
flags: [generated]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
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


async function* f([,]) {
  
};

assert.throws(Test262Error, function() {
  f(iter);
});

iter.next();
assert.sameValue(following, 0, 'Iterator was properly closed.');
