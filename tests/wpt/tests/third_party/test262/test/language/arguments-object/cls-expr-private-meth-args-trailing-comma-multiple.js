// This file was procedurally generated from the following sources:
// - src/arguments/args-trailing-comma-multiple.case
// - src/arguments/default/cls-expr-private-meth.template
/*---
description: A trailing comma should not increase the arguments.length, using multiple args (class expression private method)
esid: sec-argument-lists-runtime-semantics-argumentlistevaluation
features: [class, class-methods-private]
flags: [generated]
info: |
    9.4.4 Arguments Exotic Objects

    Most ECMAScript functions make an arguments object available to their code. Depending upon the
    characteristics of the function definition, its arguments object is either an ordinary object
    or an arguments exotic object.


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
  #method() {
    assert.sameValue(arguments.length, 2);
    assert.sameValue(arguments[0], 42);
    assert.sameValue(arguments[1], 'TC39');
    callCount = callCount + 1;
  }

  get method() {
    return this.#method;
  }
};

new C().method(42, 'TC39',);

assert.sameValue(callCount, 1, 'method invoked exactly once');
