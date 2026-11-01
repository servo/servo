async_test((t) => {
  let order = [];

  function append(n) {
    order.push(`${n}ms`);
    if (order.length == 3) {
      assert_array_equals(order, ["0ms", "1ms", "2ms"]);
      t.done();
    }
  }

  setTimeout(t.step_func(() => append(2)), 2);
  setTimeout(t.step_func(() => append(1)), 1);
  setTimeout(t.step_func(() => append(0)), 0);
}, "A 0ms timeout should not be clamped to 1ms");
