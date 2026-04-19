// This file was procedurally generated from the following sources:
// - src/class-elements/private-getter-shadowed-by-method-on-nested-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName of private getter can be shadowed on inner class by a private method (field definitions in a class declaration)
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
  get #m() { throw new Test262Error(); }

  B = class {
    method(o) {
      return o.#m();
    }

    #m() { return 'test262'; }
  }
}

let c = new C();
let innerB = new c.B();
assert.sameValue(innerB.method(innerB), 'test262');
assert.throws(TypeError, function() {
  innerB.method(c);
}, 'accessed inner class method from an object of outer class');
