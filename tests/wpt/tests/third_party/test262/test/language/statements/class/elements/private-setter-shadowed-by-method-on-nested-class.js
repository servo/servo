// This file was procedurally generated from the following sources:
// - src/class-elements/private-setter-shadowed-by-method-on-nested-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName of private setter can be shadowed on inner class by a private method (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-methods-private, class-fields-public, class]
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
  set #m(v) { this._v = v; }

  method(v) { this.#m = v; }

  B = class {
    method(o, v) {
      o.#m = v;
    }

    #m() { return 'test262'; }
  }
}

let c = new C();
let innerB = new c.B();

assert.throws(TypeError, function() {
  innerB.method(innerB, 'foo');
}, 'invalid [[Set]] operation in a private method');

c.method('outer class');
assert.sameValue(c._v, 'outer class');

assert.throws(TypeError, function() {
  innerB.method(c);
}, 'invalid access of inner class method from an object of outer class');
