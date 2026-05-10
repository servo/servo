// This file was procedurally generated from the following sources:
// - src/class-elements/private-setter-access-on-inner-arrow-function.case
// - src/class-elements/default/cls-expr.template
/*---
description: PrivateName of private setter is visible on inner arrow function of class scope (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-methods-private, class]
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

  method() {
    let arrowFunction = () => {
      this.#m = 'Test262';
    }

    arrowFunction();
  }
}

let c = new C();
c.method();
assert.sameValue(c._v, 'Test262');
let o = {};
assert.throws(TypeError, function() {
  c.method.call(o);
}, 'accessed private setter from an ordinary object');
