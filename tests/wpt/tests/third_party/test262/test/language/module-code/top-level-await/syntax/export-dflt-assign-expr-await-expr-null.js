// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-null.case
// - src/top-level-await/syntax/export-dflt-assign-expr.template
/*---
description: AwaitExpression NullLiteral (Valid syntax for top level await in export default AssignmentExpression)
esid: prod-AwaitExpression
features: [top-level-await, class]
flags: [generated, module]
info: |
    ModuleItem:
      StatementListItem[~Yield, +Await, ~Return]

    ...

    UnaryExpression[Yield, Await]
      [+Await]AwaitExpression[?Yield]

    AwaitExpression[Yield]:
      await UnaryExpression[?Yield, +Await]

    ...

    ExportDeclaration:
      export * FromClause ;
      export ExportClause FromClause ;
      export ExportClause ;
      export VariableStatement[~Yield, +Await]
      export Declaration[~Yield, +Await]
      export defaultHoistableDeclaration[~Yield, +Await, +Default]
      export defaultClassDeclaration[~Yield, +Await, +Default]
      export default [lookahead ∉ { function, async [no LineTerminator here] function, class }] AssignmentExpression [+In, ~Yield, ~Await];

    Declaration[Yield, Await]:
      HoistableDeclaration[?Yield, ?Await, ~Default]
      ClassDeclaration[?Yield, ?Await, ~Default]
      LexicalDeclaration[+In, ?Yield, ?Await]

    ClassDeclaration[Yield, Await, Default]:
      classBindingIdentifier[?Yield, ?Await] ClassTail[?Yield, ?Await]
      [+Default] class ClassTail[?Yield, ?Await]

    ClassTail[Yield, Await]:
      ClassHeritage[?Yield, ?Await]_opt { ClassBody[?Yield, ?Await]_opt }


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


export default await null;
