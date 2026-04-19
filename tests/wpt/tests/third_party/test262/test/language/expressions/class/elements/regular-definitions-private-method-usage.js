// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-usage.case
// - src/class-elements/productions/cls-expr-regular-definitions.template
/*---
description: PrivateName CallExpression usage (private method) (regular fields defintion)
esid: prod-FieldDefinition
features: [class-methods-private, class, class-fields-public]
flags: [generated]
info: |
    Updated Productions

    CallExpression[Yield, Await]:
      CoverCallExpressionAndAsyncArrowHead[?Yield, ?Await]
      SuperCall[?Yield, ?Await]
      CallExpression[?Yield, ?Await]Arguments[?Yield, ?Await]
      CallExpression[?Yield, ?Await][Expression[+In, ?Yield, ?Await]]
      CallExpression[?Yield, ?Await].IdentifierName
      CallExpression[?Yield, ?Await]TemplateLiteral[?Yield, ?Await]
      CallExpression[?Yield, ?Await].PrivateName

---*/


var C = class {
  #m() { return 'test262'; }
  method() {
    return this.#m();
  }
}

var c = new C();

assert.sameValue(c.method(), 'test262');
