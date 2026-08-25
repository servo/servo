// Test that exceptions are throw for invalid values for start and
// stop.
function testStartStop(should, node, options) {
  // Test non-finite values for start.  These should all throw a TypeError
  const nonFiniteValues = [NaN, Infinity, -Infinity];

  nonFiniteValues.forEach(time => {
      should(() => {
        node.start(time);
      }, `start(${time})`)
    .throw(TypeError);
    });

  should(() => {
    node.stop();
  }, 'Calling stop() before start()').throw(DOMException, 'InvalidStateError');

  should(() => {
    node.start(-1);
  }, 'start(-1)').throw(RangeError);

  if (options) {
    options.forEach(test => {
      should(() => {node.start(...test.args)},
             'start(' + test.args + ')').throw(test.errorType);
    });
  }

  node.start();
  should(() => {
    node.start();
  }, 'Calling start() twice').throw(DOMException, 'InvalidStateError');
  should(() => {
    node.stop(-1);
  }, 'stop(-1)').throw(RangeError);

  // Test non-finite stop times
  nonFiniteValues.forEach(time => {
      should(() => {
        node.stop(time);
      }, `stop(${time})`)
    .throw(TypeError);
    });
}

/**
 * @function
 * @param {AudioScheduledSourceNode} node - The AudioScheduledSourceNode (e.g.,
 *     ConstantSourceNode, AudioBufferSourceNode) to test.
 * @param {Array<Object>} [options] - Optional: An array of test objects for
 *     additional start() exceptions. Each object should have:
 *   - `errorType`: The expected error constructor(e.g., TypeError,
 *     RangeError).
 *   - `args`: An array of arguments to pass to the `node.start()` method.
 * @description Tests that AudioScheduledSourceNode's `start()` and `stop()`
 *   methods throw the correct exceptions for invalid input values and states,
 *   according to the Web Audio API specification. This function uses
 *   `testharness.js` assertions.
 */
const testStartStop_W3CTH = (node, options) => {
  // Test non-finite values for start. These should all throw a TypeError
  const nonFiniteValues = [NaN, Infinity, -Infinity];

  nonFiniteValues.forEach((time) => {
    assert_throws_js(TypeError, () => {
      node.start(time);
    }, `start(${time})`);
  });

  assert_throws_dom('InvalidStateError', () => {
    node.stop();
  }, 'Calling stop() before start()');

  assert_throws_js(RangeError, () => {
    node.start(-1);
  }, 'start(-1)');

  if (options) {
    options.forEach((test) => {
      assert_throws_js(test.errorType, () => {
        node.start(...test.args);
      }, `start(${test.args})`);
    });
  }

  node.start();
  assert_throws_dom('InvalidStateError', () => {
    node.start();
  }, 'Calling start() twice');
  assert_throws_js(RangeError, () => {
    node.stop(-1);
  }, 'stop(-1)');

  // Test non-finite stop times
  nonFiniteValues.forEach((time) => {
    assert_throws_js(TypeError, () => {
      node.stop(time);
    }, `stop(${time})`);
  });
}
