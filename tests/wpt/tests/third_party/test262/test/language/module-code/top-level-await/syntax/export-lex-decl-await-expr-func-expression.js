// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-func-expression.case
// - src/top-level-await/syntax/export-lex-declaration.template
/*---
description: AwaitExpression StringLiteral (Valid syntax for top level await in export LexicalDeclaration)
esid: prod-AwaitExpression
features: [top-level-await]
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
      export default[lookahead ∉ { function, async [no LineTerminator here] function, class }]AssignmentExpression[+In, ~Yield, ~Await];

    Declaration[Yield, Await]:
      HoistableDeclaration[?Yield, ?Await, ~Default]
      ClassDeclaration[?Yield, ?Await, ~Default]
      LexicalDeclaration[+In, ?Yield, ?Await]

    LexicalDeclaration[In, Yield, Await]:
      LetOrConstBindingList[?In, ?Yield, ?Await];


    ExpressionStatement[Yield, Await]:
      [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
        Expression[+In, ?Yield, ?Await];

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


export let x = await function() {};
export const y = await function() {};
