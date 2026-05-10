// This file was procedurally generated from the following sources:
// - src/class-elements/private-field-usage.case
// - src/class-elements/productions/cls-expr-wrapped-in-sc.template
/*---
description: PrivateName CallExpression usage (private field) (fields definition wrapped in semicolons)
esid: prod-FieldDefinition
features: [class-fields-private, class, class-fields-public]
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
  ;;;;
  ;;;;;;#m = 'test262';;;;;;;;
  ;;;;
  method() {
    return this.#m;
  }
}

var c = new C();

assert.sameValue(c.method(), 'test262');
