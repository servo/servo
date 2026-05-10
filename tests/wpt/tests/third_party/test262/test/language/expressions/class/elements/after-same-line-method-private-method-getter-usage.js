// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-getter-usage.case
// - src/class-elements/productions/cls-expr-after-same-line-method.template
/*---
description: PrivateName CallExpression usage (Accesor get method) (field definitions after a method in the same line)
esid: prod-FieldDefinition
features: [class-methods-private, class, class-fields-public]
flags: [generated]
includes: [propertyHelper.js]
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
  m() { return 42; } get #m() { return 'test262'; };
  method() {
    return this.#m;
  }
}

var c = new C();

assert.sameValue(c.m(), 42);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert.sameValue(c.m, C.prototype.m);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

assert.sameValue(c.method(), 'test262');
