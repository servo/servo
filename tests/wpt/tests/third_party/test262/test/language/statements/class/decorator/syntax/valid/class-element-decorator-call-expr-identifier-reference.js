// This file was procedurally generated from the following sources:
// - src/decorator/decorator-call-expr-identifier-reference.case
// - src/decorator/syntax/valid/cls-element-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorCallExpression (Valid syntax for decorator on class.)
esid: prod-ClassDeclaration
features: [class, decorators]
flags: [generated]
info: |
    ClassElement[Yield, Await] :
      DecoratorList[?Yield, ?Await]opt MethodDefinition[?Yield, ?Await]
      DecoratorList[?Yield, ?Await]opt static MethodDefinition[?Yield, ?Await]
      DecoratorList[?Yield, ?Await]opt FieldDefinition[?Yield, ?Await] ;
      DecoratorList[?Yield, ?Await]opt static FieldDefinition[?Yield, ?Await] ;
      ClassStaticBlock
      ;

    DecoratorList[Yield, Await] :
      DecoratorList[?Yield, ?Await]opt Decorator[?Yield, ?Await]

    Decorator[Yield, Await] :
      @ DecoratorMemberExpression[?Yield, ?Await]
      @ DecoratorParenthesizedExpression[?Yield, ?Await]
      @ DecoratorCallExpression[?Yield, ?Await]

    ...


    DecoratorCallExpression[Yield, Await] :
      DecoratorMemberExpression[?Yield, ?Await] Arguments[?Yield, ?Await]

    DecoratorMemberExpression[Yield, Await] :
      IdentifierReference[?Yield, ?Await]
      DecoratorMemberExpression[?Yield, ?Await] . IdentifierName
      DecoratorMemberExpression[?Yield, ?Await] . PrivateIdentifier

    IdentifierReference[Yield, Await] :
      Identifier
      [~Yield] yield
      [~Await] await

---*/
function decorator() {
  return () => {};
}
var $ = decorator;
var _ = decorator;
var \u{6F} = decorator;
var \u2118 = decorator;
var ZW_\u200C_NJ = decorator;
var ZW_\u200D_J = decorator;
var await = decorator;



class C {
  @$()
  @_()
  @\u{6F}()
  @\u2118()
  @ZW_\u200C_NJ()
  @ZW_\u200D_J()
  @await() method() {}
  @$()
  @_()
  @\u{6F}()
  @\u2118()
  @ZW_\u200C_NJ()
  @ZW_\u200D_J()
  @await() static method() {}
  @$()
  @_()
  @\u{6F}()
  @\u2118()
  @ZW_\u200C_NJ()
  @ZW_\u200D_J()
  @await() field;
  @$()
  @_()
  @\u{6F}()
  @\u2118()
  @ZW_\u200C_NJ()
  @ZW_\u200D_J()
  @await() static field;
}
