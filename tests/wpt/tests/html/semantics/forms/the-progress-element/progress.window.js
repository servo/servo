test(() => {
  const pr = document.createElement("progress");
  assert_equals(pr.value, 0);
  assert_equals(pr.position, -1);
  pr.value = 2;
  assert_equals(pr.value, 1);
  assert_equals(pr.position, 1);
}, "If value > max, then current value = max");

test(() => {
  const pr = document.createElement("progress");
  pr.value = 2;
  assert_equals(pr.value, 1);
  assert_equals(pr.position, 1);
  pr.max = 4;
  assert_equals(pr.value, 2);
  assert_equals(pr.position, 0.5);
}, "If value < max, then current value = value");
