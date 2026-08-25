// This file was procedurally generated from the following sources:
// - src/class-elements/field-definition-accessor-no-line-terminator.case
// - src/class-elements/default/cls-decl.template
/*---
description: Valid accessor FieldDefinition, ClassElementName, PropertyName Syntax (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [decorators, class]
flags: [generated]
info: |
    FieldDefinition[Yield, Await] :
      accessor [no LineTerminator here] ClassElementName[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]opt

---*/


class C {
  accessor
  $;
  static accessor
  $;

}

let c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, 'accessor'),
  "accessor doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, '$'),
  "$ doesn't appear as an own property on C prototype"
);
assert(
  Object.prototype.hasOwnProperty.call(C, 'accessor'),
  "C constructor has an own property accessor"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, '$'),
  "$ doesn't appear as an own property on C constructor"
);
assert(
  Object.prototype.hasOwnProperty.call(c, 'accessor'),
  "C instance has an own property accessor"
);
assert(
  Object.prototype.hasOwnProperty.call(c, '$'),
  "C instance has an own property $"
);
