// This file was procedurally generated from the following sources:
// - src/logical-assignment-private/and.case
// - src/logical-assignment-private/default/getter-setter.template
/*---
description: Logical-and assignment with target being a private reference (to an accessor property with getter and setter)
esid: sec-assignment-operators-runtime-semantics-evaluation
features: [class-fields-private, logical-assignment-operators]
flags: [generated]
info: |
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
      ...
      5.c. Let _setter_ be _entry_.[[Set]].
      d. Perform ? Call(_setter_, _O_, « _value_ »).


    sec-assignment-operators-runtime-semantics-evaluation
    AssignmentExpression : LeftHandSideExpression &&= AssignmentExpression
      1. Let _lref_ be the result of evaluating |LeftHandSideExpression|.
      2. Let _lval_ be ? GetValue(_lref_).
      3. Let _lbool_ be ! ToBoolean(_lval_).
      4. If _lbool_ is *false*, return _lval_.
      ...
      7. Perform ? PutValue(_lref_, _rval_).
      8. Return _rval_.
---*/


class C {
  #setterCalledWith;
  get #field() {
    return true;
  }
  set #field(value) {
    this.#setterCalledWith = value;
  }
  compoundAssignment() {
    return this.#field &&= false;
  }
  setterCalledWithValue() {
    return this.#setterCalledWith;
  }
}

const o = new C();
assert.sameValue(o.compoundAssignment(), false, "The expression should evaluate to the result");
assert.sameValue(o.setterCalledWithValue(), false, "PutValue should call the setter with the result");
