// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-next-not-callable-symbol-throw.case
// - src/async-generators/default/async-class-decl-static-method.template
/*---
description: Not-callable next value in a yield star position - symbol (Static async generator method as a ClassDeclaration element)
esid: prod-AsyncGeneratorMethod
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration]
flags: [generated, async]
info: |
    ClassElement :
      static MethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    YieldExpression: yield * AssignmentExpression
    ...
    6. Repeat
      a. If received.[[Type]] is normal, then
        ii. Let innerResult be ? Invoke(iterator, "next",
            « received.[[Value]] »).
    ...

---*/
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]() {
    return {
      next: Symbol('oi')
    };
  }
};



var callCount = 0;

class C { static async *gen() {
    callCount += 1;
    yield* obj;
      throw new Test262Error('abrupt completion closes iter');

}}

var gen = C.gen;

var iter = gen();

iter.next().then(() => {
  throw new Test262Error('Promise incorrectly fulfilled.');
}, v => {
  assert.sameValue(v.constructor, TypeError, "TypeError");

  iter.next().then(({ done, value }) => {
    assert.sameValue(done, true, 'the iterator is completed');
    assert.sameValue(value, undefined, 'value is undefined');
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
