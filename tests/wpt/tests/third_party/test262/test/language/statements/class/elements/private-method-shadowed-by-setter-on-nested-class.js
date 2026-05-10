// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-shadowed-by-setter-on-nested-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName of private method can be shadowed on inner classes by a private setter (field definitions in a class declaration)
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
  #m() { return 'outer class'; }

  method() { return this.#m(); }

  B = class {
    method(o) {
      return o.#m;
    }

    set #m(v) { this._v = v; }
  }
}

let c = new C();
let innerB = new c.B();

assert.throws(TypeError, function() {
  innerB.method(innerB);
}, '[[Get]] operation of an accessor without getter');

assert.sameValue(c.method(), 'outer class');

assert.throws(TypeError, function() {
  innerB.method(c);
}, 'access of inner class accessor from an object of outer class');
