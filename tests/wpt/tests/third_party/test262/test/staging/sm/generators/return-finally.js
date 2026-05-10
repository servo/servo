// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Return value should not be overwritten by finally block with normal execution.
info: bugzilla.mozilla.org/show_bug.cgi?id=1202134
esid: pending
---*/

// ==== single ====

var f, g, v;
f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is return
  try {
    return 42;
  } finally {
    return 43;
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is throw
  try {
    return 42;
  } finally {
    throw 43;
  }
};
var caught = false;
g = f();
try {
  v = g.next();
} catch (e) {
  assert.sameValue(e, 43);
  caught = true;
}
assert.sameValue(caught, true);

f = function*() {
  // F.[[type]] is break
  do try {
    return 42;
  } finally {
    break;
  } while (false);
  return 43;
};
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is break
  L: try {
    return 42;
  } finally {
    break L;
  }
  return 43;
};
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is continue
  do try {
    return 42;
  } finally {
    continue;
  } while (false);
  return 43;
};
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, true);

// ==== nested ====

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
    // F.[[type]] is break
    do try {
      return 43;
    } finally {
      break;
    } while (0);
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
    // F.[[type]] is break
    L: try {
      return 43;
    } finally {
      break L;
    }
  }
}
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
    // F.[[type]] is continue
    do try {
      return 43;
    } finally {
      continue;
    } while (0);
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
    // F.[[type]] is normal
    // B.[[type]] is normal
    try {
      // F.[[type]] is throw
      try {
        return 43;
      } finally {
        throw 9;
      }
    } catch (e) {
    }
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is return
  try {
    return 41;
  } finally {
    // F.[[type]] is normal
    // B.[[type]] is return
    try {
      return 42;
    } finally {
      // F.[[type]] is break
      do try {
        return 43;
      } finally {
        break;
      } while (0);
    }
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

// ==== with yield ====

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    return 42;
  } finally {
    yield 43;
  }
};
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, false);
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

// ==== throw() ====

f = function*() {
  // F.[[type]] is throw
  try {
    return 42;
  } finally {
    yield 43;
  }
};
caught = false;
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, false);
try {
  v = g.throw(44);
} catch (e) {
  assert.sameValue(e, 44);
  caught = true;
}
assert.sameValue(caught, true);

f = function*() {
  // F.[[type]] is normal
  try {
    return 42;
  } finally {
    // F.[[type]] is normal
    // B.[[type]] is throw
    try {
      yield 43;
    } catch (e) {
    }
  }
};
caught = false;
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, false);
v = g.throw(44);
assert.sameValue(v.value, 42);
assert.sameValue(v.done, true);

// ==== return() ====

f = function*() {
  // F.[[type]] is return
  try {
    return 42;
  } finally {
    yield 43;
  }
};
caught = false;
g = f();
v = g.next();
assert.sameValue(v.value, 43);
assert.sameValue(v.done, false);
v = g.return(44);
assert.sameValue(v.value, 44);
assert.sameValue(v.done, true);

f = function*() {
  // F.[[type]] is normal
  // B.[[type]] is return
  try {
    yield 42;
  } finally {
    // F.[[type]] is continue
    do try {
      return 43;
    } finally {
      continue;
    } while (0);
  }
};
caught = false;
g = f();
v = g.next();
assert.sameValue(v.value, 42);
assert.sameValue(v.done, false);
v = g.return(44);
assert.sameValue(v.value, 44);
assert.sameValue(v.done, true);
