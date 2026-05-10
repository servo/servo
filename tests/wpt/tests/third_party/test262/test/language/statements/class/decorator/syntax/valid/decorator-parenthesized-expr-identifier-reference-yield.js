// This file was procedurally generated from the following sources:
// - src/decorator/decorator-parenthesized-expr-identifier-reference-yield.case
// - src/decorator/syntax/valid/cls-decl-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorParenthesizedExpression (Valid syntax for decorator on class.)
esid: prod-ClassDeclaration
features: [class, decorators]
flags: [generated, noStrict]
info: |
    ClassDeclaration[Yield, Await, Default] :
      DecoratorList[?Yield, ?Await]opt class BindingIdentifier[?Yield, ?Await] ClassTail[?Yield, ?Await]
      [+Default] DecoratorList[?Yield, ?Await]opt class ClassTail[?Yield, ?Await]

    DecoratorList[Yield, Await] :
      DecoratorList[?Yield, ?Await]opt Decorator[?Yield, ?Await]

    Decorator[Yield, Await] :
      @ DecoratorMemberExpression[?Yield, ?Await]
      @ DecoratorParenthesizedExpression[?Yield, ?Await]
      @ DecoratorCallExpression[?Yield, ?Await]

    ...


    DecoratorParenthesizedExpression[Yield, Await] :
      ( Expression[+In, ?Yield, ?Await] )

    PrimaryExpression[Yield, Await] :
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

    IdentifierReference[Yield, Await] :
      [~Yield] yield
      ...

---*/
function yield() {}



@(yield) class C {}
