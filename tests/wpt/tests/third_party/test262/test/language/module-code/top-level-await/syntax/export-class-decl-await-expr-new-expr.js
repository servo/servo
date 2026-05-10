// This file was procedurally generated from the following sources:
// - src/top-level-await/await-expr-new-expr.case
// - src/top-level-await/syntax/export-class-decl.template
/*---
description: AwaitExpression new MemberExpression (Valid syntax for top level await in export ClassDeclaration)
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
      export default[lookahead ∉ { function, async [no LineTerminator here] function, class }]AssignmentExpression[+In, ~Yield, ~Await];

    Declaration[Yield, Await]:
      HoistableDeclaration[?Yield, ?Await, ~Default]
      ClassDeclaration[?Yield, ?Await, ~Default]
      LexicalDeclaration[+In, ?Yield, ?Await]

    ClassDeclaration[Yield, Await, Default]:
      classBindingIdentifier[?Yield, ?Await] ClassTail[?Yield, ?Await]
      [+Default] class ClassTail[?Yield, ?Await]

    ClassTail[Yield, Await]:
      ClassHeritage[?Yield, ?Await]_opt { ClassBody[?Yield, ?Await]_opt }


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


function fn() {
  return function() {};
}
// extends CallExpression with arguments
export class C extends fn(await new Promise(function(res, rej) { res(1); })) {};
