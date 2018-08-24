async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  iframe.src = "/common/blank.html";
  iframe.onload = t.step_func_done(() => {
    const win = iframe.contentWindow;
    const doc = iframe.contentDocument;
    assert_equals(win.history.state, null);
    win.history.replaceState("state", "");
    assert_equals(win.history.state, "state");
    assert_equals(doc.open(), doc);
    assert_equals(win.history.state, "state");
  });
}, "history.state is kept by document.open()");

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  iframe.src = "/common/blank.html";
  iframe.onload = t.step_func_done(() => {
    const win = iframe.contentWindow;
    const doc = iframe.contentDocument;
    assert_equals(win.history.state, null);
    win.history.replaceState("state", "");
    assert_equals(win.history.state, "state");
    assert_equals(doc.open("", "replace"), doc);
    assert_equals(win.history.state, "state");
  });
}, "history.state is kept by document.open() (with historical replace parameter set)");
