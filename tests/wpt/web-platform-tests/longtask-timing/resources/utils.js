function checkLongTaskEntry(longtask, name = 'self') {
  assert_equals(longtask.entryType, 'longtask', 'The entryType should be longtask');
  assert_equals(longtask.name, name, 'Name should be ' + name + '.');
  assert_true(Number.isInteger(longtask.duration, 'The duration should be an integer.'));
  assert_greater_than_equal(longtask.duration, 50, 'The Duration should be greater than or equal to 50.');
  assert_greater_than_equal(longtask.startTime, 0, 'The startTime should be greater than or equal to 0.');
  const currentTime = performance.now();
  assert_less_than_equal(longtask.startTime, currentTime, 'The startTime should be less than or equal to current time.');
}

function hasUnrelatedTaskName(taskName, expectedTaskName) {
  return (taskName !== expectedTaskName);
}

function busyWait() {
  const deadline = performance.now() + 100;
  while (performance.now() < deadline) {}
}
