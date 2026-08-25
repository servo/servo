// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-nested.case
// - src/top-level-await/syntax/for-of-expr.template
/*---
description: Nested AwaitExpressions (Valid syntax for top level await in for-of statements.)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    IterationStatement[Yield, Await, Return]:
      ...
      for ( [ lookahead ≠ let []Expression[~In, ?Yield, ?Await]opt ; Expression[+In, ?Yield, ?Await]opt ; Expression[+In, ?Yield, ?Await]opt ) Statement[?Yield, ?Await, ?Return]
      for ( var VariableDeclarationList[~In, ?Yield, ?Await] ; Expression[+In, ?Yield, ?Await]opt ; Expression[+In, ?Yield, ?Await]opt ) Statement[?Yield, ?Await, ?Return]
      for ( LexicalDeclaration[~In, ?Yield, ?Await] Expression[+In, ?Yield, ?Await]opt ; Expression[+In, ?Yield, ?Await]opt ) Statement[?Yield, ?Await, ?Return]
      for ( [lookahead ≠ let [] LeftHandSideExpression[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      for ( var ForBinding[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      for ( ForDeclaration[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      for ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      for ( var ForBinding[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      for ( ForDeclaration[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
      ...

    ...

    UnaryExpression[Yield, Await]
      [+Await]AwaitExpression[?Yield]

    AwaitExpression[Yield]:
      await UnaryExpression[?Yield, +Await]

    ...


    TryStatement[Yield, Await, Return]:
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]
      try Block[?Yield, ?Await, ?Return] Catch[?Yield, ?Await, ?Return] Finally[?Yield, ?Await, ?Return]

    ...

    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression[+In, ?Yield, ?Await];

---*/


var binding;

// for ( [lookahead ≠ let] LeftHandSideExpression[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}

// for ( var ForBinding[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (var binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}

// for ( ForDeclaration[?Yield, ?Await] of AssignmentExpression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (let binding of [await await await await await await await await await await await await await await await 'await']) {
  await await await await await await await await await await await await await await await 'await';
  break;
}
