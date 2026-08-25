// This file was procedurally generated from the following sources:
// - src/function-forms/eval-var-scope-syntax-err.case
// - src/function-forms/error-no-strict/gen-meth.template
/*---
description: sloppy direct eval in params introduces var (generator method in sloppy code)
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
features: [default-parameters, generators]
flags: [generated, noStrict]
info: |
    GeneratorMethod :
        * PropertyName ( StrictFormalParameters ) { GeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this GeneratorMethod is strict mode code,
       let strict be true. Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be GeneratorFunctionCreate(Method,
       StrictFormalParameters, GeneratorBody, scope, strict).
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
  *method(a = eval("var a = 42")) {
    
    callCount = callCount + 1;
  }
};

assert.throws(SyntaxError, function() {
  obj.method();
});
assert.sameValue(callCount, 0, 'generator method body not evaluated');
