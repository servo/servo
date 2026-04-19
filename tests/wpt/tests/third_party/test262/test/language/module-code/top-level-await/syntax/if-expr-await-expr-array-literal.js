// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-array-literal.case
// - src/top-level-await/syntax/if-expr.template
/*---
description: AwaitExpression ArrayLiteral (Valid syntax for top level await in an if expression position.)
esid: prod-AwaitExpression
features: [top-level-await]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    IfStatement[Yield, Await, Return]:
      if(Expression[+In, ?Yield, ?Await])Statement[?Yield, ?Await, ?Return]elseStatement[?Yield, ?Await, ?Return]
      if(Expression[+In, ?Yield, ?Await])Statement[?Yield, ?Await, ?Return]

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


if (await []) {}
