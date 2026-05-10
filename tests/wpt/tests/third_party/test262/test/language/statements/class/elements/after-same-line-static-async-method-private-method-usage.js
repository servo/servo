// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-usage.case
// - src/class-elements/productions/cls-decl-after-same-line-static-async-method.template
/*---
description: PrivateName CallExpression usage (private method) (field definitions after a static async method in the same line)
esid: prod-FieldDefinition
features: [class-methods-private, class, class-fields-public, async-functions]
flags: [generated, async]
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


class C {
  static async m() { return 42; } #m() { return 'test262'; };
  method() {
    return this.#m();
  }
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "m"),
  "m doesn't appear as an own property on the C prototype"
);

verifyProperty(C, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
}, {restore: true});

C.m().then(function(v) {
  assert.sameValue(v, 42);

  function assertions() {
    // Cover $DONE handler for async cases.
    function $DONE(error) {
      if (error) {
        throw new Test262Error('Test262:AsyncTestFailure')
      }
    }
    assert.sameValue(c.method(), 'test262');
  }

  return Promise.resolve(assertions());
}).then($DONE, $DONE);
