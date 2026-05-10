// This file was procedurally generated from the following sources:
// - src/decorator/decorator-member-expr-private-identifier.case
// - src/decorator/syntax/class-valid/cls-expr-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorMemberExpression (Valid syntax for decorator on class expression in class body.)
esid: prod-ClassExpression
features: [class, decorators]
flags: [generated]
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


    DecoratorMemberExpression[Yield, Await] :
      IdentifierReference[?Yield, ?Await]
      DecoratorMemberExpression[?Yield, ?Await] . IdentifierName
      DecoratorMemberExpression[?Yield, ?Await] . PrivateIdentifier

    PrivateIdentifier ::
      # IdentifierName

---*/


class C {
  static #$() {}
  static #_() {}
  static #\u{6F}() {}
  static #\u2118() {}
  static #ZW_\u200C_NJ() {}
  static #ZW_\u200D_J() {}
  static #yield() {}
  static #await() {}

  static {
    var D = @C.#$
    @C.#_
    @C.#\u{6F}
    @C.#\u2118
    @C.#ZW_\u200C_NJ
    @C.#ZW_\u200D_J
    @C.#yield
    @C.#await class {}
  }
};
