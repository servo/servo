// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-iter-step-err.case
// - src/dstr-binding/error/arrow-function.template
/*---
description: Error forwarding when IteratorStep returns an abrupt completion (arrow function expression)
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    ArrowFunction : ArrowParameters => ConciseBody

    [...]
    4. Let closure be FunctionCreate(Arrow, parameters, ConciseBody, scope, strict).
    [...]

    9.2.1 [[Call]] ( thisArgument, argumentsList)

    [...]
    7. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    [...]

    9.2.1.3 OrdinaryCallEvaluateBody ( F, argumentsList )

    1. Let status be FunctionDeclarationInstantiation(F, argumentsList).
    [...]

    9.2.12 FunctionDeclarationInstantiation(func, argumentsList)

    [...]
    23. Let iteratorRecord be Record {[[iterator]]:
        CreateListIterator(argumentsList), [[done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        b. Let formalStatus be IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
    [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       a. If iteratorRecord.[[done]] is false,
          i. Let next be IteratorStep(iteratorRecord.[[iterator]]).
          ii. If next is an abrupt completion, set iteratorRecord.[[done]] to
              true.
          iii. ReturnIfAbrupt(next).

---*/
var first = 0;
var second = 0;
var iter = function*() {
  first += 1;
  throw new Test262Error();
  second += 1;
}();

var f = ([...x]) => {};

assert.throws(Test262Error, function() {
  f(iter);
});

iter.next();
assert.sameValue(first, 1);
assert.sameValue(second, 0, 'Iterator is closed following abrupt completion.');
