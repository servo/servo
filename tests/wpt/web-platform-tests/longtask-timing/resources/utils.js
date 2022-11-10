function checkLongTaskEntry(longtask, name='self') {
  assert_equals(longtask.entryType, 'longtask');
  assert_equals(longtask.name, name);
  assert_true(Number.isInteger(longtask.duration));
  assert_greater_than_equal(longtask.duration, 50);
  assert_greater_than_equal(longtask.startTime, 0);
  const currentTime = performance.now();
  assert_less_than_equal(longtask.startTime, currentTime);
}

function hasUnrelatedTaskName(taskName, expectedTaskName) {
  return (taskName !== expectedTaskName);
}
