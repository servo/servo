// This file was procedurally generated from the following sources:
// - src/arguments/args-trailing-comma-single-args.case
// - src/arguments/default/async-gen-meth.template
/*---
description: A trailing comma should not increase the arguments.length, using a single arg (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [async-iteration]
flags: [generated, async]
info: |
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
var obj = {
  async *method() {
    assert.sameValue(arguments.length, 1);
    assert.sameValue(arguments[0], 42);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = obj.method;

ref(42,).next().then(() => {
    assert.sameValue(callCount, 1, 'generator method invoked exactly once');
}).then($DONE, $DONE);
