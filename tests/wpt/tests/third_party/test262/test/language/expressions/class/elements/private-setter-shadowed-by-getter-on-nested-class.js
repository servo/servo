// This file was procedurally generated from the following sources:
// - src/class-elements/private-setter-shadowed-by-getter-on-nested-class.case
// - src/class-elements/default/cls-expr.template
/*---
description: PrivateName of private setter can be shadowed on inner classes by a private getter (field definitions in a class expression)
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


var C = class {
  set #m(v) { this._v = v; }

  method(v) { this.#m = v; }

  B = class {
    method(o, v) {
      o.#m = v;
    }

    get #m() { return 'test262'; }
  }
}

let c = new C();
let innerB = new c.B();

assert.throws(TypeError, function() {
  innerB.method(innerB);
}, 'invalid [[Set]] of an acessor without setter');

c.method('outer class');
assert.sameValue(c._v, 'outer class');

assert.throws(TypeError, function() {
  innerB.method(c);
}, 'invalid access of inner class getter from an object of outer class');
