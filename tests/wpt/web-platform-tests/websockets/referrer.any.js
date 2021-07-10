// META: script=constants.js?pipe=sub

async_test(t => {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + "/referrer");
  ws.onmessage = t.step_func_done(e => {
    assert_equals(e.data, "MISSING AS PER FETCH");
    ws.close();
  });

  // Avoid timeouts in case of failure
  ws.onclose = t.unreached_func("close");
  ws.onerror = t.unreached_func("error");
}, "Ensure no Referer header is included");
