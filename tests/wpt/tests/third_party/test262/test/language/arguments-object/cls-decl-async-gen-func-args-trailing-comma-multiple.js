// This file was procedurally generated from the following sources:
// - src/arguments/args-trailing-comma-multiple.case
// - src/arguments/default/async-gen-func-decl.template
/*---
description: A trailing comma should not increase the arguments.length, using multiple args (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
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
// Stores a reference `ref` for case evaluation
async function* ref() {
  assert.sameValue(arguments.length, 2);
  assert.sameValue(arguments[0], 42);
  assert.sameValue(arguments[1], 'TC39');
  callCount = callCount + 1;
}

ref(42, 'TC39',).next().then(() => {
    assert.sameValue(callCount, 1, 'generator function invoked exactly once');
}).then($DONE, $DONE);
