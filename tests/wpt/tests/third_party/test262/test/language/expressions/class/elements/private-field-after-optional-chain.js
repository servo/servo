// This file was procedurally generated from the following sources:
// - src/class-elements/private-field-after-optional-chain.case
// - src/class-elements/default/cls-expr.template
/*---
description: OptionalChain.PrivateIdentifier is a valid syntax (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-private, optional-chaining, class]
flags: [generated]
info: |
    Updated Productions

    OptionalChain[Yield, Await] :
      `?.` `[` Expression[+In, ?Yield, ?Await] `]`
      `?.` IdentifierName
      `?.` Arguments[?Yield, ?Await]
      `?.` TemplateLiteral[?Yield, ?Await, +Tagged]
      OptionalChain[?Yield, ?Await]  `[` Expression[+In, ?Yield, ?Await] `]`
      OptionalChain[?Yield, ?Await] `.` IdentifierName
      OptionalChain[?Yield, ?Await] Arguments[?Yield, ?Await]
      OptionalChain[?Yield, ?Await] TemplateLiteral[?Yield, ?Await, +Tagged]
      OptionalChain[?Yield, ?Await] `.` PrivateIdentifier

---*/


var C = class {
  #f = 'Test262';

  method(o) {
    return o?.c.#f;
  }
}

let c = new C();
let o = {c: c};
assert.sameValue(c.method(o), 'Test262');

assert.sameValue(c.method(null), undefined);
assert.sameValue(c.method(undefined), undefined);

o = {c: new Object()};
assert.throws(TypeError, function() {
  c.method(o);
}, 'accessed private field from an ordinary object');
