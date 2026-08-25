// This file was procedurally generated from the following sources:
// - src/decorator/decorator-member-expr-decorator-member-expr.case
// - src/decorator/syntax/valid/cls-element-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorMemberExpression (Valid syntax for decorator on class.)
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


    DecoratorMemberExpression[Yield, Await] :
      IdentifierReference[?Yield, ?Await]
      DecoratorMemberExpression[?Yield, ?Await] . IdentifierName
      DecoratorMemberExpression[?Yield, ?Await] . PrivateIdentifier

---*/
let ns = {
  $() {},
  _() {},
  \u{6F}() {},
  \u2118() {},
  ZW_\u200C_NJ() {},
  ZW_\u200D_J() {},
  yield() {},
  await() {},
}



class C {
  @ns.$
  @ns._
  @ns.\u{6F}
  @ns.\u2118
  @ns.ZW_\u200C_NJ
  @ns.ZW_\u200D_J
  @ns.yield
  @ns.await method() {}
  @ns.$
  @ns._
  @ns.\u{6F}
  @ns.\u2118
  @ns.ZW_\u200C_NJ
  @ns.ZW_\u200D_J
  @ns.yield
  @ns.await static method() {}
  @ns.$
  @ns._
  @ns.\u{6F}
  @ns.\u2118
  @ns.ZW_\u200C_NJ
  @ns.ZW_\u200D_J
  @ns.yield
  @ns.await field;
  @ns.$
  @ns._
  @ns.\u{6F}
  @ns.\u2118
  @ns.ZW_\u200C_NJ
  @ns.ZW_\u200D_J
  @ns.yield
  @ns.await static field;
}
