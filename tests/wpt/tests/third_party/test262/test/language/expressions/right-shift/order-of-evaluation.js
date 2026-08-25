// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-signed-right-shift-operator-runtime-semantics-evaluation
description: Type coercion order of operations for right-shift operator
features: [Symbol]
info: |
  Evaluate lhs
  Evaluate rhs
  ToNumeric(lhs)
  ToNumeric(rhs)
---*/

function MyError() {}
var trace;

// ?GetValue(lhs) throws.
trace = "";
assert.throws(MyError, function() {
  (function() {
    trace += "1";
    throw new MyError();
  })() >> (function() {
    trace += "2";
    throw new Test262Error("should not be evaluated");
  })();
}, "?GetValue(lhs) throws.");
assert.sameValue(trace, "1", "?GetValue(lhs) throws.");

// ?GetValue(rhs) throws.
trace = "";
assert.throws(MyError, function() {
  (function() {
    trace += "1";
    return {
      valueOf: function() {
        trace += "3";
        throw new Test262Error("should not be evaluated");
      }
    };
  })() >> (function() {
    trace += "2";
    throw new MyError();
  })();
}, "?GetValue(rhs) throws.");
assert.sameValue(trace, "12", "?GetValue(rhs) throws.");

// ?ToPrimive(lhs) throws.
trace = "";
assert.throws(MyError, function() {
  (function() {
    trace += "1";
    return {
      valueOf: function() {
        trace += "3";
        throw new MyError();
      }
    };
  })() >> (function() {
    trace += "2";
    return {
      valueOf: function() {
        trace += "4";
        throw new Test262Error("should not be evaluated");
      }
    };
  })();
}, "?ToPrimive(lhs) throws.");
assert.sameValue(trace, "123", "?ToPrimive(lhs) throws.");

// ?ToPrimive(rhs) throws.
trace = "";
assert.throws(MyError, function() {
  (function() {
    trace += "1";
    return {
      valueOf: function() {
        trace += "3";
        return 1;
      }
    };
  })() >> (function() {
    trace += "2";
    return {
      valueOf: function() {
        trace += "4";
        throw new MyError();
      }
    };
  })();
}, "?ToPrimive(rhs) throws.");
assert.sameValue(trace, "1234", "?ToPrimive(rhs) throws.");

// ?ToNumeric(lhs) throws.
trace = "";
assert.throws(TypeError, function() {
  (function() {
    trace += "1";
    return {
      valueOf: function() {
        trace += "3";
        return Symbol("1");
      }
    };
  })() >> (function() {
    trace += "2";
    return {
      valueOf: function() {
        trace += "4";
        throw new Test262Error("should not be evaluated");
      }
    };
  })();
}, "?ToNumeric(lhs) throws.");
assert.sameValue(trace, "123", "?ToNumeric(lhs) throws.");

// GetValue(lhs) throws.
trace = "";
assert.throws(TypeError, function() {
  (function() {
    trace += "1";
    return {
      valueOf: function() {
        trace += "3";
        return 1;
      }
    };
  })() >> (function() {
    trace += "2";
    return {
      valueOf: function() {
        trace += "4";
        return Symbol("1");
      }
    };
  })();
}, "GetValue(lhs) throws.");
assert.sameValue(trace, "1234", "GetValue(lhs) throws.");
