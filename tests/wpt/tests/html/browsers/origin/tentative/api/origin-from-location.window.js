// META: title=`Origin.from(Location)`
// META: script=/common/get-host-info.sub.js

test(t => {
  const origin = Origin.from(window.location);
  assert_true(!!origin);
  assert_false(origin.opaque);
  assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
}, `Origin.from(window.location) returns a tuple origin.`);

async_test(t => {
  const el = document.createElement('iframe');
  el.src = "/common/blank.html";
  el.onload = t.step_func_done(_ => {
    const origin = Origin.from(el.contentWindow.location);
    assert_true(!!origin);
    assert_false(origin.opaque);
    assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
  });
  document.body.appendChild(el);
}, `Origin.from(Location) returns a tuple origin for same-origin frames.`);

async_test(t => {
  const el = document.createElement('iframe');
  el.src = get_host_info().REMOTE_ORIGIN + "/common/blank.html";
  el.onload = t.step_func_done(_ => {
    assert_throws_js(TypeError, _ => Origin.from(el.contentWindow.location));
  });
  document.body.appendChild(el);
}, `Origin.from(Location) throws for cross-origin frames.`);

async_test(t => {
  const w = window.open("/html/browsers/windows/resources/post-to-opener.html");
  window.addEventListener("message", t.step_func(e => {
    if (e.source === w) {
      const origin = Origin.from(w.location);
      assert_true(!!origin);
      assert_false(origin.opaque);
      assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
      t.done();
    }
  }));
}, `Origin.from(Location) returns a tuple origin for same-origin windows.`);

async_test(t => {
  const w = window.open(get_host_info().REMOTE_ORIGIN + "/html/browsers/windows/resources/post-to-opener.html");
  window.addEventListener("message", t.step_func(e => {
    if (e.source === w) {
      assert_throws_js(TypeError, _ => Origin.from(w.location));
      t.done();
    }
  }));
}, `Origin.from(Location) throws for cross-origin windows.`);
