// This file was procedurally generated from the following sources:
// - src/logical-assignment-private/nullish.case
// - src/logical-assignment-private/default/getter-short-circuit.template
/*---
description: Nullish-coalescing assignment with target being a private reference (to an accessor property with getter (short-circuit version))
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
      b. If _entry_.[[Set]] is *undefined*, throw a *TypeError* exception.


    sec-assignment-operators-runtime-semantics-evaluation
    AssignmentExpression : LeftHandSideExpression ??= AssignmentExpression
      1. Let _lref_ be the result of evaluating |LeftHandSideExpression|.
      2. Let _lval_ be ? GetValue(_lref_).
      3. If _lval_ is neither *undefined* nor *null*, return _lval_.
      ...
      6. Perform ? PutValue(_lref_, _rval_).
      7. Return _rval_.
---*/


function doNotCall() {
  throw new Test262Error("The right-hand side should not be evaluated");
}

class C {
  get #field() {
    return 1;
  }
  compoundAssignment() {
    return this.#field ??= doNotCall();
  }
}

const o = new C();
assert.sameValue(o.compoundAssignment(), 1, "The expression should evaluate to the short-circuit value");
