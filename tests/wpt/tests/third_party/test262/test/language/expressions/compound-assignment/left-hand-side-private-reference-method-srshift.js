// This file was procedurally generated from the following sources:
// - src/compound-assignment-private/srshift.case
// - src/compound-assignment-private/default/method.template
/*---
description: Compound signed-right-shift assignment with target being a private reference (to a private method)
esid: sec-assignment-operators-runtime-semantics-evaluation
features: [class-fields-private]
flags: [generated]
info: |
    sec-assignment-operators-runtime-semantics-evaluation
    AssignmentExpression : LeftHandSideExpression AssignmentOperator AssignmentExpression
      1. Let _lref_ be the result of evaluating |LeftHandSideExpression|.
      2. Let _lval_ be ? GetValue(_lref_).
      ...
      7. Let _r_ be ApplyStringOrNumericBinaryOperator(_lval_, _opText_, _rval_).
      8. Perform ? PutValue(_lref_, _r_).
      9. Return _r_.

    sec-property-accessors-runtime-semantics-evaluation
    MemberExpression : MemberExpression `.` PrivateIdentifier

      1. Let _baseReference_ be the result of evaluating |MemberExpression|.
      2. Let _baseValue_ be ? GetValue(_baseReference_).
      3. Let _fieldNameString_ be the StringValue of |PrivateIdentifier|.
      4. Return ! MakePrivateReference(_baseValue_, _fieldNameString_).

    PutValue (V, W)
      ...
      5.b. If IsPrivateReference(_V_) is *true*, then
        i. Return ? PrivateSet(_baseObj_, _V_.[[ReferencedName]], _W_).

    PrivateSet (O, P, value)
      ...
      4. Else if _entry_.[[Kind]] is ~method~, then
        a. Throw a *TypeError* exception.

---*/


class C {
  #privateMethod() {}
  compoundAssignment() {
    return this.#privateMethod >>= 1;
  }
}

const o = new C();
assert.throws(TypeError, () => o.compoundAssignment(), "PutValue throws when storing the result in a method private reference");
