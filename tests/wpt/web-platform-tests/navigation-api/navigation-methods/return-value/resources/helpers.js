window.assertReturnValue = (result, w = window) => {
  assert_equals(Object.getPrototypeOf(result), w.Object.prototype, "result object must be from the right realm");
  assert_array_equals(Reflect.ownKeys(result), ["committed", "finished"]);
  assert_true(result.committed instanceof w.Promise);
  assert_true(result.finished instanceof w.Promise);
  assert_not_equals(result.committed, result.finished);
};

window.assertNeverSettles = (t, result, w = window) => {
  assertReturnValue(result, w);
  result.committed.then(
    t.unreached_func("committed must not fulfill"),
    t.unreached_func("committed must not reject")
  );

  result.finished.then(
    t.unreached_func("finished must not fulfill"),
    t.unreached_func("finished must not reject")
  );
};

window.assertBothFulfill = async (t, result, expected, w = window) => {
  assertReturnValue(result, w);

  // Don't use await here so that we can catch out-of-order settlements.
  let committedValue;
  result.committed.then(
    t.step_func(v => { committedValue = v; }),
    t.unreached_func("committed must not reject")
  );

  const finishedValue = await result.finished;

  assert_not_equals(committedValue, undefined, "committed must fulfill before finished");
  assert_equals(finishedValue, committedValue, "committed and finished must fulfill with the same value");
  assert_true(finishedValue instanceof w.NavigationHistoryEntry, "fulfillment value must be a NavigationHistoryEntry");
  assert_equals(finishedValue, expected);
};

window.assertCommittedFulfillsFinishedRejectsExactly = async (t, result, expectedEntry, expectedRejection, w = window) => {
  assertReturnValue(result, w);

  // Don't use await here so that we can catch out-of-order settlements.
  let committedValue;
  result.committed.then(
    t.step_func(v => { committedValue = v; }),
    t.unreached_func("committed must not reject")
  );

  await promise_rejects_exactly(t, expectedRejection, result.finished);

  assert_not_equals(committedValue, undefined, "committed must fulfill before finished rejects");
  assert_true(committedValue instanceof w.NavigationHistoryEntry, "fulfillment value must be a NavigationHistoryEntry");
  assert_equals(committedValue, expectedEntry);
};

window.assertCommittedFulfillsFinishedRejectsDOM = async (t, result, expectedEntry, expectedDOMExceptionCode, w = window, domExceptionConstructor = w.DOMException, navigationHistoryEntryConstuctor = w.NavigationHistoryEntry) => {
  assertReturnValue(result, w);

  // Don't use await here so that we can catch out-of-order settlements.
  let committedValue;
  result.committed.then(
    t.step_func(v => { committedValue = v; }),
    t.unreached_func("committed must not reject")
  );

  await promise_rejects_dom(t, expectedDOMExceptionCode, domExceptionConstructor, result.finished);

  assert_not_equals(committedValue, undefined, "committed must fulfill before finished rejects");
  assert_true(committedValue instanceof navigationHistoryEntryConstuctor, "fulfillment value must be an NavigationHistoryEntry");
  assert_equals(committedValue, expectedEntry);
};

window.assertBothRejectDOM = async (t, result, expectedDOMExceptionCode, w = window, domExceptionConstructor = w.DOMException) => {
  assertReturnValue(result, w);

  // Don't use await here so that we can catch out-of-order settlements.
  let committedReason, finishedReason;
  await Promise.all([
    result.committed.then(
      t.unreached_func("committed must not fulfill"),
      t.step_func(r => { committedReason = r; })
    ),
    result.finished.then(
      t.unreached_func("finished must not fulfill"),
      t.step_func(r => { finishedReason = r; })
    )
  ]);

  assert_equals(committedReason, finishedReason, "committed and finished must reject with the same value");
  assert_throws_dom(expectedDOMExceptionCode, domExceptionConstructor, () => { throw committedReason; });
};
