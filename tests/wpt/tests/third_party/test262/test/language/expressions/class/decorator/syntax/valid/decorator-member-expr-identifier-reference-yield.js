// This file was procedurally generated from the following sources:
// - src/decorator/decorator-member-expr-identifier-reference-yield.case
// - src/decorator/syntax/valid/cls-expr-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorMemberExpression (Valid syntax for decorator on class expression)
esid: prod-ClassExpression
features: [class, decorators]
flags: [generated, noStrict]
info: |
    ClassExpression[Yield, Await] :
      DecoratorList[?Yield, ?Await]opt class BindingIdentifier[?Yield, ?Await]opt ClassTail[?Yield, ?Await]

    DecoratorList[Yield, Await] :
      DecoratorList[?Yield, ?Await]opt Decorator[?Yield, ?Await]

    Decorator[Yield, Await] :
      @ DecoratorMemberExpression[?Yield, ?Await]
      @ DecoratorParenthesizedExpression[?Yield, ?Await]
      @ DecoratorCallExpression[?Yield, ?Await]

    ...


    IdentifierReference[Yield, Await] :
      [~Yield] yield
      ...

---*/
function yield() {}



var C = @yield class {};
