// This file was procedurally generated from the following sources:
// - src/class-elements/private-setter-on-nested-class.case
// - src/class-elements/default/cls-expr.template
/*---
description: PrivateName of private setter is available on inner classes (field definitions in a class expression)
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

  B = class {
    method(o, v) {
      o.#m = v;
    }
  }
}

let c = new C();
let innerB = new c.B();
innerB.method(c, 'test262');
assert.sameValue(c._v, 'test262');
