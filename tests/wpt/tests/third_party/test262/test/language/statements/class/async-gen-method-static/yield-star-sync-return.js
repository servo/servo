// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-sync-return.case
// - src/async-generators/default/async-class-decl-static-method.template
/*---
description: execution order for yield* with sync iterator and return() (Static async generator method as a ClassDeclaration element)
esid: prod-AsyncGeneratorMethod
features: [Symbol.iterator, async-iteration, Symbol.asyncIterator]
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
      ...
      c. Else,
        i. Assert: received.[[Type]] is return.
        ii. Let return be ? GetMethod(iterator, "return").
        iii. If return is undefined, return Completion(received).
        iv. Let innerReturnResult be ? Call(return, iterator,
            « received.[[Value]] »).
        v. If generatorKind is async, then set innerReturnResult to
           ? Await(innerReturnResult).
        ...
        vii. Let done be ? IteratorComplete(innerReturnResult).
        viii. If done is true, then
             1. Let value be ? IteratorValue(innerReturnResult).
             2. Return Completion{[[Type]]: return, [[Value]]: value,
                [[Target]]: empty}.
        ix. Let received be GeneratorYield(innerResult).

    %AsyncFromSyncIteratorPrototype%.return ( value )

    5. Let return be GetMethod(syncIterator, "return").
    ...
    ...
    8. Let returnResult be Call(return, syncIterator, « value »).
    ...
    11. Let returnValue be IteratorValue(returnResult).
    ..
    13. Let returnDone be IteratorComplete(returnResult).
    ...
    16. Perform ! Call(valueWrapperCapability.[[Resolve]], undefined, « returnValue »).
    ...
    18. Set onFulfilled.[[Done]] to returnDone.
    19. Perform ! PerformPromiseThen(valueWrapperCapability.[[Promise]],
        onFulfilled, undefined, promiseCapability).
    ...

---*/
var log = [];
var obj = {
  [Symbol.iterator]() {
    var returnCount = 0;
    return {
      name: "syncIterator",
      get next() {
        log.push({ name: "get next" });
        return function() {
          return {
            value: "next-value-1",
            done: false
          };
        };
      },
      get return() {
        log.push({
          name: "get return",
          thisValue: this
        });
        return function() {
          log.push({
            name: "call return",
            thisValue: this,
            args: [...arguments]
          });

          returnCount++;
          if (returnCount == 1) {
            return {
              name: "return-result-1",
              get value() {
                log.push({
                  name: "get return value (1)",
                  thisValue: this
                });
                return "return-value-1";
              },
              get done() {
                log.push({
                  name: "get return done (1)",
                  thisValue: this
                });
                return false;
              }
            };
          }

          return {
            name: "return-result-2",
            get value() {
              log.push({
                name: "get return value (2)",
                thisValue: this
              });
              return "return-value-2";
            },
            get done() {
              log.push({
                name: "get return done (2)",
                thisValue: this
              });
              return true;
            }
          };
        };
      }
    };
  }
};



var callCount = 0;

class C { static async *gen() {
    callCount += 1;
    log.push({ name: "before yield*" });
      yield* obj;

}}

var gen = C.gen;

var iter = gen();

assert.sameValue(log.length, 0, "log.length");

iter.next().then(v => {
  assert.sameValue(log[0].name, "before yield*");

  assert.sameValue(log[1].name, "get next");

  assert.sameValue(v.value, "next-value-1");
  assert.sameValue(v.done, false);

  assert.sameValue(log.length, 2, "log.length");

  iter.return("return-arg-1").then(v => {
    assert.sameValue(log[2].name, "get return");
    assert.sameValue(log[2].thisValue.name, "syncIterator", "get return thisValue");

    assert.sameValue(log[3].name, "call return");
    assert.sameValue(log[3].thisValue.name, "syncIterator", "return thisValue");
    assert.sameValue(log[3].args.length, 1, "return args.length");
    assert.sameValue(log[3].args[0], "return-arg-1", "return args[0]");

    assert.sameValue(log[4].name, "get return done (1)");
    assert.sameValue(log[4].thisValue.name, "return-result-1", "get return done thisValue");

    assert.sameValue(log[5].name, "get return value (1)");
    assert.sameValue(log[5].thisValue.name, "return-result-1", "get return value thisValue");

    assert.sameValue(v.value, "return-value-1");
    assert.sameValue(v.done, false);

    assert.sameValue(log.length, 6, "log.length");

    iter.return().then(v => {
      assert.sameValue(log[6].name, "get return");
      assert.sameValue(log[6].thisValue.name, "syncIterator", "get return thisValue");

      assert.sameValue(log[7].name, "call return");
      assert.sameValue(log[7].thisValue.name, "syncIterator", "get return thisValue");
      assert.sameValue(log[7].args.length, 1, "return args.length");
      assert.sameValue(log[7].args[0], undefined, "return args[0]");

      assert.sameValue(log[8].name, "get return done (2)");
      assert.sameValue(log[8].thisValue.name, "return-result-2", "get return done thisValue");

      assert.sameValue(log[9].name, "get return value (2)");
      assert.sameValue(log[9].thisValue.name, "return-result-2", "get return value thisValue");

      assert.sameValue(v.value, "return-value-2");
      assert.sameValue(v.done, true);

      assert.sameValue(log.length, 10, "log.length");
    }).then($DONE, $DONE);
  }).catch($DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
