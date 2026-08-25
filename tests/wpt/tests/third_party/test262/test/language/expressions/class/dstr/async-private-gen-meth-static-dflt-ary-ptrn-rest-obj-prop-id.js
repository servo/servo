// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-obj-prop-id.case
// - src/dstr-binding/default/cls-expr-async-private-gen-meth-static-dflt.template
/*---
description: Rest element containing an object binding pattern (private static class expression async generator method (default parameter))
esid: sec-class-definitions-runtime-semantics-evaluation
features: [class, class-static-methods-private, async-iteration]
flags: [generated, async]
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
        b. Else,
           Let status be the result of performing PropertyDefinitionEvaluation
           for m with arguments F and false.
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

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]
---*/
let length = "outer";


var callCount = 0;
var C = class {
  static async * #method([...{ 0: v, 1: w, 2: x, 3: y, length: z }] = [7, 8, 9]) {
    assert.sameValue(v, 7);
    assert.sameValue(w, 8);
    assert.sameValue(x, 9);
    assert.sameValue(y, undefined);
    assert.sameValue(z, 3);

    assert.sameValue(length, "outer", "the length prop is not set as a binding name");
    callCount = callCount + 1;
  }

  static get method() {
    return this.#method;
  }
};

C.method().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');    
}).then($DONE, $DONE);
