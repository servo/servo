// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-private-field-optional-chaining.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName after '?.' is valid syntax (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-private, optional-chaining, class]
flags: [generated]
info: |
    Updated Productions

    OptionalChain[Yield, Await]:
      `?.` `[` Expression[+In, ?Yield, ?Await] `]`
      `?.` IdentifierName
      `?.` Arguments[?Yield, ?Await]
      `?.` TemplateLiteral[?Yield, ?Await, +Tagged]
      `?.` PrivateIdentifier

---*/


class C {
  #m = 'test262';

  static access(obj) {
    return obj?.#m;
  }
}

let c = new C();

assert.sameValue(C.access(c), 'test262');

assert.sameValue(C.access(null), undefined);
assert.sameValue(C.access(undefined), undefined);

assert.throws(TypeError, function() {
  C.access({});
}, 'accessed private field from an ordinary object');

