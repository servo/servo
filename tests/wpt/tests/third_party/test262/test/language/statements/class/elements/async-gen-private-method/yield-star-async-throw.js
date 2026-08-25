// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-async-throw.case
// - src/async-generators/default/async-class-decl-private-method.template
/*---
description: execution order for yield* with async iterator and throw() (Async Generator method as a ClassDeclaration element)
esid: prod-AsyncGeneratorPrivateMethod
features: [async-iteration, Symbol.asyncIterator, class-methods-private]
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
    6. Repeat
      ...
      b. Else if received.[[Type]] is throw, then
        i. Let throw be ? GetMethod(iterator, "throw").
        ii. If throw is not undefined, then
          1. Let innerResult be ? Call(throw, iterator, « received.[[Value]] »).
          2. If generatorKind is async, then set innerResult to ? Await(innerResult).
          ...
          5. Let done be ? IteratorComplete(innerResult).
          6. If done is true, then
            a. Let resultValue be Return ? IteratorValue(innerResult).
            b. If generatorKind is async, then set resultValue to ? Await(resultValue).
            c. Return resultValue.
          7. If generatorKind is async, then let received be AsyncGeneratorYield(? IteratorValue(innerResult)).
      ...

    AsyncGeneratorYield ( value )

    ...
    8. Return ! AsyncGeneratorResolve(generator, value, false).
    ...

---*/
var log = [];
var obj = {
  [Symbol.asyncIterator]() {
    var throwCount = 0;
    return {
      name: "asyncIterator",
      get next() {
        log.push({ name: "get next" });
        return function() {
          return {
            value: "next-value-1",
            done: false
          };
        };
      },
      get throw() {
        log.push({
          name: "get throw",
          thisValue: this
        });
        return function() {
          log.push({
            name: "call throw",
            thisValue: this,
            args: [...arguments]
          });

          throwCount++;
          if (throwCount == 1) {
            return {
              name: "throw-promise-1",
              get then() {
                log.push({
                  name: "get throw then (1)",
                  thisValue: this
                });
                return function(resolve) {
                  log.push({
                    name: "call throw then (1)",
                    thisValue: this,
                    args: [...arguments]
                  });

                  resolve({
                    name: "throw-result-1",
                    get value() {
                      log.push({
                        name: "get throw value (1)",
                        thisValue: this
                      });
                      return "throw-value-1";
                    },
                    get done() {
                      log.push({
                        name: "get throw done (1)",
                        thisValue: this
                      });
                      return false;
                    }
                  });
                };
              }
            };
          }

          return {
            name: "throw-promise-2",
            get then() {
              log.push({
                name: "get throw then (2)",
                thisValue: this
              });
              return function(resolve) {
                log.push({
                  name: "call throw then (2)",
                  thisValue: this,
                  args: [...arguments]
                });

                resolve({
                  name: "throw-result-2",
                  get value() {
                    log.push({
                      name: "get throw value (2)",
                      thisValue: this
                    });
                    return "throw-value-2";
                  },
                  get done() {
                    log.push({
                      name: "get throw done (2)",
                      thisValue: this
                    });
                    return true;
                  }
                });
              };
            }
          };
        };
      }
    };
  }
};



var callCount = 0;

class C {
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

iter.next().then(v => {
  assert.sameValue(log[0].name, "before yield*");

  assert.sameValue(log[1].name, "get next");

  assert.sameValue(v.value, "next-value-1");
  assert.sameValue(v.done, false);

  assert.sameValue(log.length, 2, "log.length");

  iter.throw("throw-arg-1").then(v => {
    assert.sameValue(log[2].name, "get throw");
    assert.sameValue(log[2].thisValue.name, "asyncIterator", "get throw thisValue");

    assert.sameValue(log[3].name, "call throw");
    assert.sameValue(log[3].thisValue.name, "asyncIterator", "throw thisValue");
    assert.sameValue(log[3].args.length, 1, "throw args.length");
    assert.sameValue(log[3].args[0], "throw-arg-1", "throw args[0]");

    assert.sameValue(log[4].name, "get throw then (1)");
    assert.sameValue(log[4].thisValue.name, "throw-promise-1", "get throw thisValue");

    assert.sameValue(log[5].name, "call throw then (1)");
    assert.sameValue(log[5].thisValue.name, "throw-promise-1", "throw thisValue");
    assert.sameValue(log[5].args.length, 2, "throw then args.length");
    assert.sameValue(typeof log[5].args[0], "function", "throw then args[0]");
    assert.sameValue(typeof log[5].args[1], "function", "throw then args[1]");

    assert.sameValue(log[6].name, "get throw done (1)");
    assert.sameValue(log[6].thisValue.name, "throw-result-1", "get throw done thisValue");

    assert.sameValue(log[7].name, "get throw value (1)");
    assert.sameValue(log[7].thisValue.name, "throw-result-1", "get throw value thisValue");

    assert.sameValue(v.value, "throw-value-1");
    assert.sameValue(v.done, false);

    assert.sameValue(log.length, 8, "log.length");

    iter.throw("throw-arg-2").then(v => {
      assert.sameValue(log[8].name, "get throw");
      assert.sameValue(log[8].thisValue.name, "asyncIterator", "get throw thisValue");

      assert.sameValue(log[9].name, "call throw");
      assert.sameValue(log[9].thisValue.name, "asyncIterator", "throw thisValue");
      assert.sameValue(log[9].args.length, 1, "throw args.length");
      assert.sameValue(log[9].args[0], "throw-arg-2", "throw args[0]");

      assert.sameValue(log[10].name, "get throw then (2)");
      assert.sameValue(log[10].thisValue.name, "throw-promise-2", "get throw thisValue");

      assert.sameValue(log[11].name, "call throw then (2)");
      assert.sameValue(log[11].thisValue.name, "throw-promise-2", "throw thisValue");
      assert.sameValue(log[11].args.length, 2, "throw then args.length");
      assert.sameValue(typeof log[11].args[0], "function", "throw then args[0]");
      assert.sameValue(typeof log[11].args[1], "function", "throw then args[1]");

      assert.sameValue(log[12].name, "get throw done (2)");
      assert.sameValue(log[12].thisValue.name, "throw-result-2", "get throw done thisValue");

      assert.sameValue(log[13].name, "get throw value (2)");
      assert.sameValue(log[13].thisValue.name, "throw-result-2", "get throw value thisValue");

      assert.sameValue(log[14].name, "after yield*");
      assert.sameValue(log[14].value, "throw-value-2");

      assert.sameValue(v.value, "return-value");
      assert.sameValue(v.done, true);

      assert.sameValue(log.length, 15, "log.length");
    }).then($DONE, $DONE);
  }).catch($DONE);
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
