// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-async-next.case
// - src/async-generators/default/async-class-decl-static-method.template
/*---
description: Execution order for yield* with async iterator and next() (Static async generator method as a ClassDeclaration element)
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


    
---*/
var log = [];
var obj = {
  get [Symbol.iterator]() {
    log.push({ name: "get [Symbol.iterator]" });
  },
  get [Symbol.asyncIterator]() {
    log.push({
      name: "get [Symbol.asyncIterator]",
      thisValue: this
    });
    return function() {
      log.push({
        name: "call [Symbol.asyncIterator]",
        thisValue: this,
        args: [...arguments]
      });
      var nextCount = 0;
      return {
        name: "asyncIterator",
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
                name: "next-promise-1",
                get then() {
                  log.push({
                    name: "get next then (1)",
                    thisValue: this
                  });
                  return function(resolve) {
                    log.push({
                      name: "call next then (1)",
                      thisValue: this,
                      args: [...arguments]
                    });

                    resolve({
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
                    });
                  };
                }
              };
            }

            return {
              name: "next-promise-2",
              get then() {
                log.push({
                  name: "get next then (2)",
                  thisValue: this
                });
                return function(resolve) {
                  log.push({
                    name: "call next then (2)",
                    thisValue: this,
                    args: [...arguments]
                  });

                  resolve({
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
                  });
                };
              }
            };
          };
        }
      };
    };
  }
};



var callCount = 0;

class C { static async *gen() {
    callCount += 1;
    log.push({ name: "before yield*" });
      var v = yield* obj;
      log.push({
        name: "after yield*",
        value: v
      });
      return "return-value";

}}

var gen = C.gen;

var iter = gen();

assert.sameValue(log.length, 0, "log.length");

iter.next("next-arg-1").then(v => {
  assert.sameValue(log[0].name, "before yield*");

  assert.sameValue(log[1].name, "get [Symbol.asyncIterator]");
  assert.sameValue(log[1].thisValue, obj, "get [Symbol.asyncIterator] thisValue");

  assert.sameValue(log[2].name, "call [Symbol.asyncIterator]");
  assert.sameValue(log[2].thisValue, obj, "[Symbol.asyncIterator] thisValue");
  assert.sameValue(log[2].args.length, 0, "[Symbol.asyncIterator] args.length");

  assert.sameValue(log[3].name, "get next");
  assert.sameValue(log[3].thisValue.name, "asyncIterator", "get next thisValue");

  assert.sameValue(log[4].name, "call next");
  assert.sameValue(log[4].thisValue.name, "asyncIterator", "next thisValue");
  assert.sameValue(log[4].args.length, 1, "next args.length");
  assert.sameValue(log[4].args[0], undefined, "next args[0]");

  assert.sameValue(log[5].name, "get next then (1)");
  assert.sameValue(log[5].thisValue.name, "next-promise-1", "get next then thisValue");

  assert.sameValue(log[6].name, "call next then (1)");
  assert.sameValue(log[6].thisValue.name, "next-promise-1", "next then thisValue");
  assert.sameValue(log[6].args.length, 2, "next then args.length");
  assert.sameValue(typeof log[6].args[0], "function", "next then args[0]");
  assert.sameValue(typeof log[6].args[1], "function", "next then args[1]");

  assert.sameValue(log[7].name, "get next done (1)");
  assert.sameValue(log[7].thisValue.name, "next-result-1", "get next done thisValue");

  assert.sameValue(log[8].name, "get next value (1)");
  assert.sameValue(log[8].thisValue.name, "next-result-1", "get next value thisValue");

  assert.sameValue(v.value, "next-value-1");
  assert.sameValue(v.done, false);

  assert.sameValue(log.length, 9, "log.length");

  iter.next("next-arg-2").then(v => {
    assert.sameValue(log[9].name, "call next");
    assert.sameValue(log[9].thisValue.name, "asyncIterator", "next thisValue");
    assert.sameValue(log[9].args.length, 1, "next args.length");
    assert.sameValue(log[9].args[0], "next-arg-2", "next args[0]");

    assert.sameValue(log[10].name, "get next then (2)");
    assert.sameValue(log[10].thisValue.name, "next-promise-2", "get next then thisValue");

    assert.sameValue(log[11].name, "call next then (2)");
    assert.sameValue(log[11].thisValue.name, "next-promise-2", "next then thisValue");
    assert.sameValue(log[11].args.length, 2, "next then args.length");
    assert.sameValue(typeof log[11].args[0], "function", "next then args[0]");
    assert.sameValue(typeof log[11].args[1], "function", "next then args[1]");

    assert.sameValue(log[12].name, "get next done (2)");
    assert.sameValue(log[12].thisValue.name, "next-result-2", "get next done thisValue");

    assert.sameValue(log[13].name, "get next value (2)");
    assert.sameValue(log[13].thisValue.name, "next-result-2", "get next value thisValue");

    assert.sameValue(log[14].name, "after yield*");
    assert.sameValue(log[14].value, "next-value-2");

    assert.sameValue(v.value, "return-value");
    assert.sameValue(v.done, true);

    assert.sameValue(log.length, 15, "log.length");
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
