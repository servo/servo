// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js

const executor_path = '/common/dispatcher/executor.html?pipe=';
const cross_origin = get_host_info().OTHER_ORIGIN;
const coep_require_corp_header =
    '|header(Cross-Origin-Embedder-Policy,require-corp)';
const corp_cross_origin_header =
    '|header(Cross-Origin-Resource-Policy,cross-origin)';

promise_test(async t => {
  assert_true(crossOriginIsolated, 'Is main frame crossOriginIsolated?');

  const reply_token = token();
  const iframe_token = token();

  const iframe = document.createElement('iframe');
  iframe.src = cross_origin + executor_path + coep_require_corp_header +
      corp_cross_origin_header + `&uuid=${iframe_token}`;
  document.body.appendChild(iframe);

  send(iframe_token, `send('${reply_token}', 'Iframe loaded');`);
  assert_equals(await receive(reply_token), 'Iframe loaded');

  send(iframe_token, `
    window.popup = window.open();
    send('${reply_token}', popup === null);
  `);
  assert_equals(await receive(reply_token), 'false', 'Is popup handle null?');

  send(
      iframe_token,
      `send('${reply_token}', popup.window.crossOriginIsolated);`);
  assert_equals(
      await receive(reply_token), 'false', 'Is popup crossOriginIsolated?');
});
