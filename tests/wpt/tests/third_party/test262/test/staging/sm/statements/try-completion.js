// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  try block should return try value if finally returned normally
info: bugzilla.mozilla.org/show_bug.cgi?id=819125
esid: pending
---*/

function expectTryValue(code, isUndefined) {
  assert.sameValue(eval(code), isUndefined ? undefined : 'try');
}

function expectCatchValue(code, isUndefined) {
  assert.sameValue(eval(code), isUndefined ? undefined : 'catch');
}

function expectFinallyValue(code, isUndefined) {
  assert.sameValue(eval(code), isUndefined ? undefined : 'finally');
}

// ==== finally: normal ====

// try: normal
// finally: normal
expectTryValue(`
try {
  'try';
} finally {
  'finally';
}
`);

// try: normal without value
// finally: normal
expectTryValue(`
try {
} finally {
  'finally';
}
`, true);

// try: break
// finally: normal
expectTryValue(`
while (true) {
  try {
    'try';
    break;
  } finally {
    'finally';
  }
}
`);

// try: break without value
// finally: normal
expectTryValue(`
while (true) {
  try {
    break;
  } finally {
    'finally';
  }
}
`, true);

// try: continue
// finally: normal
expectTryValue(`
do {
  try {
    'try';
    continue;
  } finally {
    'finally';
  }
} while (false);
`);

// try: continue without value
// finally: normal
expectTryValue(`
do {
  try {
    continue;
  } finally {
    'finally';
  }
} while (false);
`, true);

// try: throw
// catch: normal
// finally: normal
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
  'catch';
} finally {
  'finally';
}
`);

// try: throw
// catch: normal
// finally: normal
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
  'catch';
} finally {
  'finally';
}
`);

// try: throw
// catch: normal without value
// finally: normal
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
} finally {
  'finally';
}
`, true);

// try: throw
// catch: normal without value
// finally: normal
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
} finally {
  'finally';
}
`, true);

// try: throw
// catch: break
// finally: normal
expectCatchValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
    break;
  } finally {
    'finally';
  }
}
`);

// try: throw
// catch: break without value
// finally: normal
expectCatchValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    break;
  } finally {
    'finally';
  }
}
`, true);

// try: throw
// catch: continue
// finally: normal
expectCatchValue(`
do {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
    continue;
  } finally {
    'finally';
  }
} while (false);
`);

// try: throw
// catch: continue without value
// finally: normal
expectCatchValue(`
do {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    continue;
  } finally {
    'finally';
  }
} while (false);
`, true);

// ==== finally: break ====

// try: normal
// finally: break
expectFinallyValue(`
while (true) {
  try {
    'try';
  } finally {
    'finally';
    break;
  }
}
`);

// try: normal
// finally: break without value
expectFinallyValue(`
while (true) {
  try {
    'try';
  } finally {
    break;
  }
}
`, true);

// try: break
// finally: break
expectFinallyValue(`
while (true) {
  try {
    'try';
    break;
  } finally {
    'finally';
    break;
  }
}
`);

// try: break
// finally: break without value
expectFinallyValue(`
while (true) {
  try {
    'try';
    break;
  } finally {
    break;
  }
}
`, true);

// try: continue
// finally: break
expectFinallyValue(`
do {
  try {
    'try';
    continue;
  } finally {
    'finally';
    break;
  }
} while (false);
`);

// try: continue
// finally: break without value
expectFinallyValue(`
do {
  try {
    'try';
    continue;
  } finally {
    break;
  }
} while (false);
`, true);

// try: throw
// catch: normal
// finally: break
expectFinallyValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
  } finally {
    'finally';
    break;
  }
}
`, false);

// try: throw
// catch: normal
// finally: break without value
expectFinallyValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
  } finally {
    break;
  }
}
`, true);

// ==== finally: continue ====

// try: normal
// finally: continue
expectFinallyValue(`
do {
  try {
    'try';
  } finally {
    'finally';
    continue;
  }
} while (false);
`);

// try: normal
// finally: continue without value
expectFinallyValue(`
do {
  try {
    'try';
  } finally {
    continue;
  }
} while (false);
`, true);

// try: break
// finally: continue
expectFinallyValue(`
do {
  try {
    'try';
    break;
  } finally {
    'finally';
    continue;
  }
} while (false);
`);

// try: break
// finally: continue without value
expectFinallyValue(`
do {
  try {
    'try';
    break;
  } finally {
    continue;
  }
} while (false);
`, true);

// try: continue
// finally: continue
expectFinallyValue(`
do {
  try {
    'try';
    continue;
  } finally {
    'finally';
    continue;
  }
} while (false);
`);

// try: continue
// finally: continue without value
expectFinallyValue(`
do {
  try {
    'try';
    continue;
  } finally {
    continue;
  }
} while (false);
`, true);

// ==== without finally ====

// try: throw
// catch: normal
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
  'catch';
}
`);

// try: throw
// catch: normal without value
expectCatchValue(`
try {
  'try';
  throw 'exception';
} catch (e) {
}
`, true);

// try: throw
// catch: break
expectCatchValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
    break;
  }
}
`);

// try: throw
// catch: break without value
expectCatchValue(`
while (true) {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    break;
  }
}
`, true);

// try: throw
// catch: continue
expectCatchValue(`
do {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    'catch';
    continue;
  }
} while (false);
`);

// try: throw
// catch: continue without value
expectCatchValue(`
do {
  try {
    'try';
    throw 'exception';
  } catch (e) {
    continue;
  }
} while (false);
`, true);
