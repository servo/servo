// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-iter-val-array-prototype.case
// - src/dstr-binding/default/cls-expr-meth.template
/*---
description: Array destructuring uses overriden Array.prototype[Symbol.iterator] (class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [Symbol.iterator, generators, destructuring-binding]
flags: [generated]
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
           i. Let status be the result of performing
              PropertyDefinitionEvaluation for m with arguments proto and
              false.
        [...]

    14.3.8 Runtime Semantics: DefineMethod

    MethodDefinition : PropertyName ( StrictFormalParameters ) { FunctionBody }

    [...]
    6. Let closure be FunctionCreate(kind, StrictFormalParameters, FunctionBody,
       scope, strict). If functionPrototype was passed as a parameter then pass its
       value as the functionPrototype optional argument of FunctionCreate.
    [...]

    9.2.1 [[Call]] ( thisArgument, argumentsList)

    [...]
    7. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    [...]

    9.2.1.3 OrdinaryCallEvaluateBody ( F, argumentsList )

    1. Let status be FunctionDeclarationInstantiation(F, argumentsList).
    [...]

    9.2.12 FunctionDeclarationInstantiation(func, argumentsList)

    [...]
    23. Let iteratorRecord be Record {[[iterator]]:
        CreateListIterator(argumentsList), [[done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        b. Let formalStatus be IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
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
var C = class {
  method([x, y, z]) {
    assert.sameValue(x, 1);
    assert.sameValue(y, 2);
    assert.sameValue(z, 42);
    callCount = callCount + 1;
  }
};

new C().method([1, 2, 3]);
assert.sameValue(callCount, 1, 'method invoked exactly once');
