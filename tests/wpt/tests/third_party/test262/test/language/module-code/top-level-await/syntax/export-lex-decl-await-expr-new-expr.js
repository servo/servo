// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-new-expr.case
// - src/top-level-await/syntax/export-lex-declaration.template
/*---
description: AwaitExpression new MemberExpression (Valid syntax for top level await in export LexicalDeclaration)
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


    LeftHandSideExpression[Yield, Await]:
      NewExpression[?Yield, ?Await]
      CallExpression[?Yield, ?Await]

    NewExpression[Yield, Await]:
      MemberExpression[?Yield, ?Await]
      new NewExpression[?Yield, ?Await]

    MemberExpression[Yield, Await]:
      ...
      new MemberExpression[?Yield, ?Await] Arguments[?Yield, ?Await]

---*/


export let x = await new Promise(function(res, rej) { res(1); });
export const y = await new Promise(function(res, rej) { res(1); });
