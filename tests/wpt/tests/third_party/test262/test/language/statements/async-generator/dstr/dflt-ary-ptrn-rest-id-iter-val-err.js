// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-iter-val-err.case
// - src/dstr-binding/error/async-gen-func-decl-dflt.template
/*---
description: Error forwarding when IteratorValue returns an abrupt completion (async generator function declaration (default parameter))
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [Symbol.iterator, async-iteration]
flags: [generated]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       c. Let nextValue be IteratorValue(next).
       d. If nextValue is an abrupt completion, set iteratorRecord.[[done]] to
          true.
       e. ReturnIfAbrupt(nextValue).

---*/
var poisonedValue = Object.defineProperty({}, 'value', {
  get: function() {
    throw new Test262Error();
  }
});
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedValue;
    }
  };
};


async function* f([...x] = iter) {
  
};

assert.throws(Test262Error, function() {
  f();
});
