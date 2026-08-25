// This file was procedurally generated from the following sources:
// - src/arguments/args-trailing-comma-single-args.case
// - src/arguments/default/cls-expr-async-gen-meth-static.template
/*---
description: A trailing comma should not increase the arguments.length, using a single arg (static class expression async generator method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [async-iteration]
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


    Trailing comma in the arguments list

    Left-Hand-Side Expressions

    Arguments :
        ( )
        ( ArgumentList )
        ( ArgumentList , )

    ArgumentList :
        AssignmentExpression
        ... AssignmentExpression
        ArgumentList , AssignmentExpression
        ArgumentList , ... AssignmentExpression
---*/


var callCount = 0;
var C = class {
  static async *method() {
    assert.sameValue(arguments.length, 1);
    assert.sameValue(arguments[0], 42);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = C.method;

ref(42,).next().then(() => {
    assert.sameValue(callCount, 1, 'method invoked exactly once');
}).then($DONE, $DONE);
