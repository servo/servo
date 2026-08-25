// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-iter-step-err.case
// - src/dstr-binding/error/cls-expr-async-gen-meth-dflt.template
/*---
description: Error forwarding when IteratorStep returns an abrupt completion (class expression async generator method (default parameter))
esid: sec-class-definitions-runtime-semantics-evaluation
features: [generators, async-iteration]
flags: [generated]
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


var C = class {
  async *method([...x] = iter) {
    
  }
};

var method = C.prototype.method;

assert.throws(Test262Error, function() {
  method();
});

iter.next();
assert.sameValue(first, 1);
assert.sameValue(second, 0, 'Iterator is closed following abrupt completion.');
