// This file was procedurally generated from the following sources:
// - src/class-elements/private-getter-shadowed-by-field-on-nested-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName of private getter can be shadowed on inner classes by a private field (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-methods-private, class-fields-private, class-fields-public, class]
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


class C {
  get #m() { return 'outer class'; }

  method() { return this.#m; }

  B = class {
    method(o) {
      return o.#m;
    }

    #m = 'test262';
  }
}

let c = new C();
let innerB = new c.B();
assert.sameValue(innerB.method(innerB), 'test262');
assert.sameValue(c.method(), 'outer class');
assert.throws(TypeError, function() {
  innerB.method(c);
}, 'accessed inner class field from an object of outer class');
