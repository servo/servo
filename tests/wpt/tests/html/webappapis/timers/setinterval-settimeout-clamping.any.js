// META: title=setTimeout and setInterval sequencing is correct even with 0 timeout
// META: spec=https://html.spec.whatwg.org/#run-steps-after-a-timeout
async_test(t => {
  let done = false;
  const id = setInterval(() => {
    done = true;
  }, 0);
  t.add_cleanup(() => clearInterval(id));

  setTimeout(t.step_func(() => {
    assert_true(done);
    t.done();
  }), 0);
}, "setInterval(0) before setTimeout(0)");

async_test(t => {
  let done = false;
  setTimeout(() => {
    done = true;
  }, 0);

  const id = setInterval(t.step_func(() => {
    assert_true(done);
    t.done();
  }), 0);
  t.add_cleanup(() => clearInterval(id));
}, "setTimeout(0) before setInterval(0)");
