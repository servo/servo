// This file was procedurally generated from the following sources:
// - src/function-forms/eval-var-scope-syntax-err.case
// - src/function-forms/error-no-strict/meth.template
/*---
description: sloppy direct eval in params introduces var (method in sloppy code)
esid: sec-runtime-semantics-definemethod
features: [default-parameters]
flags: [generated, noStrict]
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

    
    Runtime Semantics: IteratorBindingInitialization
    FormalParameter : BindingElement

    1. Return the result of performing IteratorBindingInitialization for BindingElement with arguments iteratorRecord and environment.

---*/

var callCount = 0;
var obj = {
  method(a = eval("var a = 42")) {
    
    callCount = callCount + 1;
  }
};

assert.throws(SyntaxError, function() {
  obj.method();
});
assert.sameValue(callCount, 0, 'method body not evaluated');
