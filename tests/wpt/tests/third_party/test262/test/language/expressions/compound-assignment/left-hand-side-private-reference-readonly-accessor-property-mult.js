// This file was procedurally generated from the following sources:
// - src/compound-assignment-private/mult.case
// - src/compound-assignment-private/default/getter.template
/*---
description: Compound multiplication assignment with target being a private reference (to an accessor property with getter)
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
      5.a. Assert: _entry_.[[Kind]] is ~accessor~.
      b. If _entry_.[[Set]] is *undefined*, throw a *TypeError* exception.

---*/


class C {
  get #field() {
    return 1;
  }
  compoundAssignment() {
    return this.#field *= 1;
  }
}

const o = new C();
assert.throws(TypeError, () => o.compoundAssignment(), "PutValue throws when storing the result if no setter");
