async_test(t => {
  const run_result = 'test_frame_OK';
  const blob_contents = '<!doctype html>\n<meta charset="utf-8">\n' +
    '<script>window.test_result = "' + run_result + '";</script>';
  const blob = new Blob([blob_contents], {type: 'text/html'});
  const url = URL.createObjectURL(blob);

  const frame = document.createElement('iframe');
  frame.setAttribute('src', url);
  frame.setAttribute('style', 'display:none;');
  document.body.appendChild(frame);
  URL.revokeObjectURL(url);

  frame.onload = t.step_func_done(() => {
    assert_equals(frame.contentWindow.test_result, run_result);
  });
}, 'Fetching a blob URL immediately before revoking it works in an iframe.');

async_test(t => {
  const run_result = 'test_frame_OK';
  const blob_contents = '<!doctype html>\n<meta charset="utf-8">\n' +
    '<script>window.test_result = "' + run_result + '";</script>';
  const blob = new Blob([blob_contents], {type: 'text/html'});
  const url = URL.createObjectURL(blob);

  const frame = document.createElement('iframe');
  frame.setAttribute('src', '/common/blank.html');
  frame.setAttribute('style', 'display:none;');
  document.body.appendChild(frame);

  frame.onload = t.step_func(() => {
    frame.contentWindow.location = url;
    URL.revokeObjectURL(url);
    frame.onload = t.step_func_done(() => {
      assert_equals(frame.contentWindow.test_result, run_result);
    });
  });
}, 'Fetching a blob URL immediately before revoking it works in an iframe navigation.');

async_test(t => {
  const run_result = 'test_script_OK';
  const blob_contents = 'window.script_test_result = "' + run_result + '";';
  const blob = new Blob([blob_contents]);
  const url = URL.createObjectURL(blob);

  const e = document.createElement('script');
  e.setAttribute('src', url);
  e.onload = t.step_func_done(() => {
    assert_equals(window.script_test_result, run_result);
  });

  document.body.appendChild(e);
  URL.revokeObjectURL(url);
}, 'Fetching a blob URL immediately before revoking it works in <script> tags.');
