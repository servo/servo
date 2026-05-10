// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generatorvalidate
description: >
  A TypeError should be thrown if the generator is in the "executing" state,
  and the generator should be marked as "completed"
info: |
  25.3.3.1 GeneratorStart

  [...]
  4. Set the code evaluation state of genContext such that when evaluation is
     resumed for that execution context the following steps will be performed:
     a. Let result be the result of evaluating generatorBody.
     b. Assert: If we return here, the generator either threw an exception or
        performed either an implicit or explicit return.
     c. Remove genContext from the execution context stack and restore the
        execution context that is at the top of the execution context stack as
        the running execution context.
     d. Set generator.[[GeneratorState]] to "completed".
     [...]

  25.3.3.4 GeneratorResumeAbrupt

  1. Let state be ? GeneratorValidate(generator).

  25.3.3.2 GeneratorValidate

  1. If Type(generator) is not Object, throw a TypeError exception.
  2. If generator does not have a [[GeneratorState]] internal slot, throw a
     TypeError exception.
  3. Assert: generator also has a [[GeneratorContext]] internal slot.
  4. Let state be generator.[[GeneratorState]].
  5. If state is "executing", throw a TypeError exception.
features: [generators]
---*/

var iter, result;
function* g() {
  iter.return(42);
}

iter = g();
assert.throws(TypeError, function() {
  iter.next();
});

result = iter.next();

assert.sameValue(typeof result, 'object');
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
