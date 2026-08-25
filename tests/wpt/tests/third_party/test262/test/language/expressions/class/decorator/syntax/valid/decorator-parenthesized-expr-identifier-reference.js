// This file was procedurally generated from the following sources:
// - src/decorator/decorator-parenthesized-expr-identifier-reference.case
// - src/decorator/syntax/valid/cls-expr-decorators-valid-syntax.template
/*---
description: Decorator @ DecoratorParenthesizedExpression (Valid syntax for decorator on class expression)
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

---*/
function $() {}
function _() {}
function \u{6F}() {}
function \u2118() {}
function ZW_\u200C_NJ() {}
function ZW_\u200D_J() {}
function await() {}



var C = @($)
@(_)
@(\u{6F})
@(\u2118)
@(ZW_\u200C_NJ)
@(ZW_\u200D_J)
@(await) class {};
