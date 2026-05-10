// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-iter-val-array-prototype.case
// - src/dstr-binding/default/cls-decl-async-private-gen-meth-static.template
/*---
description: Array destructuring uses overriden Array.prototype[Symbol.iterator] (private static class expression async generator method)
esid: sec-runtime-semantics-bindingclassdeclarationevaluation
features: [Symbol.iterator, generators, class, class-static-methods-private, async-iteration]
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
        b. Else,
           Let status be the result of performing PropertyDefinitionEvaluation for
           m with arguments F and false.
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


    Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializer_opt

    1. Let bindingId be StringValue of BindingIdentifier.
    2. Let lhs be ? ResolveBinding(bindingId, environment).
    3. If iteratorRecord.[[Done]] is false, then
        a. Let next be IteratorStep(iteratorRecord).
        b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
        c. ReturnIfAbrupt(next).
        d. If next is false, set iteratorRecord.[[Done]] to true.
        e. Else,
            i. Let v be IteratorValue(next).
            ii. If v is an abrupt completion, set iteratorRecord.[[Done]] to true.
            iii. ReturnIfAbrupt(v).
    [...]
    7. Return InitializeReferencedBinding(lhs, v).

---*/
Array.prototype[Symbol.iterator] = function* () {
    if (this.length > 0) {
        yield this[0];
    }
    if (this.length > 1) {
        yield this[1];
    }
    if (this.length > 2) {
        yield 42;
    }
};


var callCount = 0;
class C {
  static async * #method([x, y, z]) {
    assert.sameValue(x, 1);
    assert.sameValue(y, 2);
    assert.sameValue(z, 42);
    callCount = callCount + 1;
  }

  static get method() {
    return this.#method;
  }
};

C.method([1, 2, 3]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');    
}).then($DONE, $DONE);
