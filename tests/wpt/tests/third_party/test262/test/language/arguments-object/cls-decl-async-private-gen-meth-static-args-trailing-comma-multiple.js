// This file was procedurally generated from the following sources:
// - src/arguments/args-trailing-comma-multiple.case
// - src/arguments/default/cls-decl-async-private-gen-meth-static.template
/*---
description: A trailing comma should not increase the arguments.length, using multiple args (static class expression private generator method)
esid: sec-argument-lists-runtime-semantics-argumentlistevaluation
features: [async-iteration, class, class-static-methods-private]
flags: [generated, async]
info: |
    Arguments :
      ( )
      ( ArgumentList )
      ( ArgumentList , )

    ArgumentList :
      AssignmentExpression
      ... AssignmentExpression
      ArgumentList , AssignmentExpression
      ArgumentList , ... AssignmentExpression


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
class C {
  static async * #method() {
    assert.sameValue(arguments.length, 2);
    assert.sameValue(arguments[0], 42);
    assert.sameValue(arguments[1], 'TC39');
    callCount = callCount + 1;
  }

  static get method() {
      return this.#method;
  }
}

C.method(42, 'TC39',).next().then(() => {
    assert.sameValue(callCount, 1, 'method invoked exactly once');
}).then($DONE, $DONE);
