async_test(t => {
  const img = new Image();
  t.add_cleanup(() => img.remove());
  img.onloadstart = img.onprogress = img.onloadend = t.unreached_func("progress event fired");
  img.onload = t.step_func_done(e => {
    assert_true(e instanceof Event);
    assert_false(e instanceof ProgressEvent);
  });
  img.src = "/images/rrgg-256x256.png";
  document.body.append(img);
}, "<img> does not support ProgressEvent or loadstart/progress/loadend");

test(t => {
  assert_equals(document.body.onloadend, undefined);
  assert_equals(window.onloadend, undefined);
}, "onloadend is not exposed");
