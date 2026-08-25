// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/cls-expr-async-gen-meth-static-dflt.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (static class expression async generator method (default parameter))
esid: sec-class-definitions-runtime-semantics-evaluation
features: [object-rest, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
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

---*/
var count = 0;


var callCount = 0;
var C = class {
  static async *method({...x} = { get v() { count++; return 2; } }) {
    assert.sameValue(count, 1);

    verifyProperty(x, "v", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 2
    });
    callCount = callCount + 1;
  }
};

C.method().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');    
}).then($DONE, $DONE);
