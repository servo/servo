// This file was procedurally generated from the following sources:
// - src/class-elements/computed-name-toprimitive-symbol.case
// - src/class-elements/default/cls-decl.template
/*---
description: ToPrimitive evaluation in the ComputedPropertyName (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, Symbol.toPrimitive, Symbol, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    Runtime Semantics: ClassDefinitionEvaluation

    ...
    27. For each ClassElement e in order from elements
      a. If IsStatic of me is false, then
        i. Let fields be the result of performing ClassElementEvaluation for e with arguments proto and false.
      b. Else,
        i. Let fields be the result of performing ClassElementEvaluation for e with arguments F and false.
      c. If fields is an abrupt completion, then
        i. Set the running execution context's LexicalEnvironment to lex.
        ii. Set the running execution context's PrivateNameEnvironment to outerPrivateEnvironment.
        iii. Return Completion(status).
    ...

    Runtime Semantics: ClassElementEvaluation

    ClassElement: FieldDefinition;
      Return ClassFieldDefinitionEvaluation of FieldDefinition with parameter false and object.

    Runtime Semantics: ClassFieldDefinitionEvaluation
      With parameters isStatic and homeObject.

    1. Let fieldName be the result of evaluating ClassElementName.
    2. ReturnIfAbrupt(fieldName).
    ...

    Runtime Semantics: Evaluation
      ComputedPropertyName: [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
    3. Return ? ToPropertyKey(propName).

---*/
var s1 = Symbol();
var s2 = Symbol();
var s3 = Symbol();
var err = function() { throw new Test262Error(); };
var obj1 = {
  [Symbol.toPrimitive]: function() { return s1; },
  toString: err,
  valueOf: err
};

var obj2 = {
  toString: function() { return s2; },
  valueOf: err
};

var obj3 = {
  toString: undefined,
  valueOf: function() { return s3; }
};



class C {
  [obj1] = 42;
  [obj2] = 43;
  [obj3] = 44;
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, s1),
  "s1 doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, s1),
  "s1 doesn't appear as an own property on C constructor"
);

verifyProperty(c, s1, {
  value: 42,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, s2),
  "s2 doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, s2),
  "s2 doesn't appear as an own property on C constructor"
);

verifyProperty(c, s2, {
  value: 43,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, s3),
  "s3 doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, s3),
  "s3 doesn't appear as an own property on C constructor"
);

verifyProperty(c, s3, {
  value: 44,
  enumerable: true,
  writable: true,
  configurable: true
});
