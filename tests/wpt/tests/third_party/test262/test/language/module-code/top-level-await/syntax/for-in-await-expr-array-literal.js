// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-array-literal.case
// - src/top-level-await/syntax/for-in-expr.template
/*---
description: AwaitExpression ArrayLiteral (Valid syntax for top level await in for-in statements.)
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


    PrimaryExpression[Yield, Await]:
      this
      IdentifierReference[?Yield, ?Await]
      Literal
      ArrayLiteral[?Yield, ?Await]
      ObjectLiteral[?Yield, ?Await]
      FunctionExpression
      ClassExpression[?Yield, ?Await]
      GeneratorExpression
      AsyncFunctionExpression
      AsyncGeneratorExpression
      RegularExpressionLiteral
      TemplateLiteral[?Yield, ?Await, ~Tagged]
      CoverParenthesizedExpressionAndArrowParameterList[?Yield, ?Await]

---*/


var binding;

// for ( [lookahead ≠ let [] LeftHandSideExpression[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (binding in [await []]) {
  await [];
  break;
}

// for ( var ForBinding[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (var binding in [await []]) {
  await [];
  break;
}

// for ( ForDeclaration[?Yield, ?Await] in Expression[+In, ?Yield, ?Await] ) Statement[?Yield, ?Await, ?Return]
for (let binding in [await []]) {
  await [];
  break;
}
