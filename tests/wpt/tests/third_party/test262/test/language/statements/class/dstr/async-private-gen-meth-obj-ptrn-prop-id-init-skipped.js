// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init-skipped.case
// - src/dstr-binding/default/cls-decl-async-private-gen-meth.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (private class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [class, class-methods-private, async-iteration]
flags: [generated, async]
info: |
    ClassDeclaration : class BindingIdentifier ClassTail

    1. Let className be StringValue of BindingIdentifier.
    2. Let value be the result of ClassDefinitionEvaluation of ClassTail with
       argument className.
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


    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    BindingElement : BindingPattern Initializeropt

    [...]
    3. If Initializer is present and v is undefined, then
    [...]
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}


var callCount = 0;
class C {
  async * #method({ s: t = counter(), u: v = counter(), w: x = counter(), y: z = counter() }) {
    assert.sameValue(t, null);
    assert.sameValue(v, 0);
    assert.sameValue(x, false);
    assert.sameValue(z, '');
    assert.sameValue(initCount, 0);

    assert.throws(ReferenceError, function() {
      s;
    });
    assert.throws(ReferenceError, function() {
      u;
    });
    assert.throws(ReferenceError, function() {
      w;
    });
    assert.throws(ReferenceError, function() {
      y;
    });
    callCount = callCount + 1;
  }

  get method() {
    return this.#method;
  }
};

new C().method({ s: null, u: 0, w: false, y: '' }).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');    
}).then($DONE, $DONE);
