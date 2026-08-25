// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-sync-next.case
// - src/async-generators/default/async-class-expr-private-method.template
/*---
description: execution order for yield* with sync iterator and next() (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [Symbol.iterator, async-iteration, Symbol.asyncIterator, class-methods-private]
flags: [generated, async]
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    YieldExpression: yield * AssignmentExpression

    ...
    2. Let value be ? GetValue(exprRef).
    3. Let generatorKind be ! GetGeneratorKind().
    4. Let iterator be ? GetIterator(value, generatorKind).
    5. Let received be NormalCompletion(undefined).
    6. Repeat
      a. If received.[[Type]] is normal, then
        i. Let innerResult be ? IteratorNext(iterator, received.[[Value]]).
        ii. Let innerResult be ? Invoke(iterator, "next",
            « received.[[Value]] »).
        iii. If generatorKind is async, then set innerResult to
             ? Await(innerResult).
        ...
        v. Let done be ? IteratorComplete(innerResult).
        vi. If done is true, then
           1. Return ? IteratorValue(innerResult).
        vii. Let received be GeneratorYield(innerResult).
      ...

    GetIterator ( obj [ , hint ] )

    ...
    3. If hint is async,
      a. Set method to ? GetMethod(obj, @@asyncIterator).
      b. If method is undefined,
        i. Let syncMethod be ? GetMethod(obj, @@iterator).
        ii. Let syncIterator be ? Call(syncMethod, obj).
        iii. Return ? CreateAsyncFromSyncIterator(syncIterator).
    ...

    %AsyncFromSyncIteratorPrototype%.next ( value )

    ...
    5. Let nextResult be IteratorNext(syncIterator, value).
    ...
    7. Let nextValue be IteratorValue(nextResult).
    ...
    9. Let nextDone be IteratorComplete(nextResult).
    ...
    12. Perform ! Call(valueWrapperCapability.[[Resolve]], undefined,
        « nextValue »).
    ...
    14. Set onFulfilled.[[Done]] to nextDone.
    15. Perform ! PerformPromiseThen(valueWrapperCapability.[[Promise]],
        onFulfilled, undefined, promiseCapability).
    ...

    Async Iterator Value Unwrap Functions

    1. Return ! CreateIterResultObject(value, F.[[Done]]).

---*/
var log = [];
var obj = {
  get [Symbol.iterator]() {
    log.push({
      name: "get [Symbol.iterator]",
      thisValue: this
    });
    return function() {
      log.push({
        name: "call [Symbol.iterator]",
        thisValue: this,
        args: [...arguments]
      });
      var nextCount = 0;
      return {
        name: "syncIterator",
        get next() {
          log.push({
            name: "get next",
            thisValue: this
          });
          return function() {
            log.push({
              name: "call next",
              thisValue: this,
              args: [...arguments]
            });

            nextCount++;
            if (nextCount == 1) {
              return {
                name: "next-result-1",
                get value() {
                  log.push({
                    name: "get next value (1)",
                    thisValue: this
                  });
                  return "next-value-1";
                },
                get done() {
                  log.push({
                    name: "get next done (1)",
                    thisValue: this
                  });
                  return false;
                }
              };
            }

            return {
              name: "next-result-2",
              get value() {
                log.push({
                  name: "get next value (2)",
                  thisValue: this
                });
                return "next-value-2";
              },
              get done() {
                log.push({
                  name: "get next done (2)",
                  thisValue: this
                });
                return true;
              }
            };
          };
        }
      };
    };
  },
  get [Symbol.asyncIterator]() {
    log.push({ name: "get [Symbol.asyncIterator]" });
    return null;
  }
};



var callCount = 0;

var C = class {
    async *#gen() {
        callCount += 1;
        log.push({ name: "before yield*" });
          var v = yield* obj;
          log.push({
            name: "after yield*",
            value: v
          });
          return "return-value";

    }
    get gen() { return this.#gen; }
}

const c = new C();

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);

var iter = c.gen();

assert.sameValue(log.length, 0, "log.length");

iter.next("next-arg-1").then(v => {
  assert.sameValue(log[0].name, "before yield*");

  assert.sameValue(log[1].name, "get [Symbol.asyncIterator]");

  assert.sameValue(log[2].name, "get [Symbol.iterator]");
  assert.sameValue(log[2].thisValue, obj, "get [Symbol.iterator] thisValue");

  assert.sameValue(log[3].name, "call [Symbol.iterator]");
  assert.sameValue(log[3].thisValue, obj, "[Symbol.iterator] thisValue");
  assert.sameValue(log[3].args.length, 0, "[Symbol.iterator] args.length");

  assert.sameValue(log[4].name, "get next");
  assert.sameValue(log[4].thisValue.name, "syncIterator", "get next thisValue");

  assert.sameValue(log[5].name, "call next");
  assert.sameValue(log[5].thisValue.name, "syncIterator", "next thisValue");
  assert.sameValue(log[5].args.length, 1, "next args.length");
  assert.sameValue(log[5].args[0], undefined, "next args[0]");

  assert.sameValue(log[6].name, "get next done (1)");
  assert.sameValue(log[6].thisValue.name, "next-result-1", "get next done thisValue");

  assert.sameValue(log[7].name, "get next value (1)");
  assert.sameValue(log[7].thisValue.name, "next-result-1", "get next value thisValue");

  assert.sameValue(v.value, "next-value-1");
  assert.sameValue(v.done, false);

  assert.sameValue(log.length, 8, "log.length");

  iter.next("next-arg-2").then(v => {
    assert.sameValue(log[8].name, "call next");
    assert.sameValue(log[8].thisValue.name, "syncIterator", "next thisValue");
    assert.sameValue(log[8].args.length, 1, "next args.length");
    assert.sameValue(log[8].args[0], "next-arg-2", "next args[0]");

    assert.sameValue(log[9].name, "get next done (2)");
    assert.sameValue(log[9].thisValue.name, "next-result-2", "get next done thisValue");

    assert.sameValue(log[10].name, "get next value (2)");
    assert.sameValue(log[10].thisValue.name, "next-result-2", "get next value thisValue");

    assert.sameValue(log[11].name, "after yield*");
    assert.sameValue(log[11].value, "next-value-2");

    assert.sameValue(v.value, "return-value");
    assert.sameValue(v.done, true);

    assert.sameValue(log.length, 12, "log.length");
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);

// Test the private fields do not appear as properties after set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);
