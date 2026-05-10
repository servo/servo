// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-init-id.case
// - src/dstr-binding/default/meth-dflt.template
/*---
description: Rest element (identifier) does not support initializer (method (default parameter))
esid: sec-runtime-semantics-definemethod
features: [destructuring-binding, default-parameters]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    MethodDefinition : PropertyName ( StrictFormalParameters ) { FunctionBody }

    [...]
    6. Let closure be FunctionCreate(kind, StrictFormalParameters,
       FunctionBody, scope, strict). If functionPrototype was passed as a
       parameter then pass its value as the functionPrototype optional argument
       of FunctionCreate.
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

    13.3.3 Destructuring Binding Patterns
    ArrayBindingPattern[Yield] :
        [ Elisionopt BindingRestElement[?Yield]opt ]
        [ BindingElementList[?Yield] ]
        [ BindingElementList[?Yield] , Elisionopt BindingRestElement[?Yield]opt ]
---*/
$DONOTEVALUATE();

var callCount = 0;
var obj = {
  method([...x = []] = []) {
    
    callCount = callCount + 1;
  }
};

obj.method();
assert.sameValue(callCount, 1, 'method invoked exactly once');
