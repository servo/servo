// META: title=`Origin.from(Window)`
// META: script=/common/get-host-info.sub.js

test(t => {
  const origin = Origin.from(window);
  assert_true(!!origin);
  assert_false(origin.opaque);
  assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
}, `Origin.from(window) returns a tuple origin.`);

async_test(t => {
  const el = document.createElement('iframe');
  el.src = "/common/blank.html";
  el.onload = t.step_func_done(_ => {
    const origin = Origin.from(el.contentWindow);
    assert_true(!!origin);
    assert_false(origin.opaque);
    assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
  });
  document.body.appendChild(el);
}, `Origin.from(Window) returns a tuple origin for same-origin frames.`);

async_test(t => {
  const el = document.createElement('iframe');
  el.src = get_host_info().REMOTE_ORIGIN + "/common/blank.html";
  el.onload = t.step_func_done(_ => {
    assert_throws_js(TypeError, _ => Origin.from(el.contentWindow));
  });
  document.body.appendChild(el);
}, `Origin.from(Window) throws for cross-origin frames.`);

async_test(t => {
  const el = document.createElement('iframe');
  el.src = "/common/blank.html";
  el.sandbox = "allow-scripts";
  el.onload = t.step_func_done(_ => {
    assert_throws_js(TypeError, _ => Origin.from(el.contentWindow));
    t.done();
  });
  document.body.appendChild(el);
}, `Origin.from(Window) throws for sandboxed frames.`);

async_test(t => {
  const w = window.open("/html/browsers/windows/resources/post-to-opener.html");
  window.addEventListener("message", t.step_func(e => {
    if (e.source === w) {
      const origin = Origin.from(w);
      assert_true(!!origin);
      assert_false(origin.opaque);
      assert_true(origin.isSameOrigin(Origin.from(get_host_info().ORIGIN)));
      t.done();
    }
  }));
}, `Origin.from(Window) returns a tuple origin for same-origin windows.`);

async_test(t => {
  const w = window.open(get_host_info().REMOTE_ORIGIN + "/html/browsers/windows/resources/post-to-opener.html");
  window.addEventListener("message", t.step_func(e => {
    if (e.source === w) {
      assert_throws_js(TypeError, _ => Origin.from(w));
      t.done();
    }
  }));
}, `Origin.from(Window) throws for cross-origin windows.`);

async_test(t => {
  const html = `<script>
  const originA = Origin.from(globalThis);
  const originB = Origin.from(globalThis);
  window.parent.postMessage({
    "isOpaque": originA.opaque,
    "sameOrigin": originA.isSameOrigin(originB),
  }, "*");
</script>
`;

  const el = document.createElement('iframe');
  el.src = `data:text/html;base64,${btoa(html)}`;
  window.addEventListener("message", t.step_func(e => {
    if (e.source === el.contentWindow) {
      assert_true(e.data.isOpaque, "Origin should be opaque.");
      assert_true(e.data.sameOrigin, "Window's origin should be same-origin with itself.");
      t.done();
    }
  }));
  document.body.appendChild(el);
}, `Origin.from(Window) returns an opaque origin for a data URL source.`);
