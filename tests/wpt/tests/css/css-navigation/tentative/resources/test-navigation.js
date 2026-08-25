function matches(elm, property) {
  return getComputedStyle(elm).getPropertyValue(property) == 'matched' ? 1 : 0;
}

function get_match_result(match_params) {
  let result = new Array();
  for (param of match_params) {
    result.push(matches(param[0], param[1]));
  }
  return result;
}

function assert_match_result(result, expectation, stage, match_params) {
  assert_equals(result.length, expectation.length, 'Array length mismatch');
  for (let idx = 0; idx < result.length; idx++) {
    assert_equals(result[idx], expectation[idx],
                  `${match_params[idx][0].id} ${
                      match_params[idx][1]} mismatch, unexpectedly ${
                      result[idx]} at stage ${stage}`);
  }
}

async function test_navigation(t, test_id, operation, onnavigate_expectations,
                               committed_expectations, match_params) {
  let nav = Promise.withResolvers();
  let precommit = Promise.withResolvers();
  let handler = Promise.withResolvers();
  navigation.onnavigate =
      (event) => {
        nav.resolve();
        event.intercept({
          async precommitHandler() {
            precommit.resolve();
            await new Promise(r => t.step_timeout(r, 0));
          },
          async handler() {
            handler.resolve();
            await new Promise(resolve => requestAnimationFrame(resolve));
          }
        })
      }

  // Prevent the test from failing asserts at certain stages, since that would
  // interfere with the testharness framework, so that the test would hang
  // instead of failing. Store the result, and check them later, at a safe
  // stage.
  let onnavigate_result,
  precommit_result;

  const empty_result = new Array(match_params.length);
  empty_result.fill(0);

  assert_match_result(get_match_result(match_params), empty_result,
                      `${test_id} before`, match_params);
  operation();
  await nav.promise;
  onnavigate_result = get_match_result(match_params);
  await precommit.promise;
  precommit_result = get_match_result(match_params);
  await navigation.transition.committed;
  assert_match_result(onnavigate_result, onnavigate_expectations,
                      `${test_id} onnavigate`, match_params);
  assert_match_result(precommit_result, onnavigate_result,
                      `${test_id} precommit`, match_params);
  const result = get_match_result(match_params);
  assert_match_result(result, committed_expectations, `${test_id} committed`,
                      match_params);
  const previous_result = result;
  await handler.promise;
  assert_match_result(get_match_result(match_params), previous_result,
                      `${test_id} handler`, match_params);
  await new Promise(r => navigation.onnavigatesuccess = () =>
                        t.step_timeout(r, 0));

  assert_match_result(get_match_result(match_params), empty_result,
                      `${test_id} after`, match_params);
}
