// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js

const executor_path = '/common/dispatcher/executor.html?pipe=';
const cross_origin = get_host_info().OTHER_ORIGIN;
const same_origin = get_host_info().ORIGIN;
const coep_require_corp_header =
    '|header(Cross-Origin-Embedder-Policy,require-corp)';
const corp_cross_origin_header =
    '|header(Cross-Origin-Resource-Policy,cross-origin)';
const coop_restrict_properties_header =
    '|header(Cross-Origin-Opener-Policy,restrict-properties)';

function iframePopupAboutBlankTest(
    origin, {expectedCrossOriginIsolated}, description) {
  promise_test(async t => {
    assert_true(crossOriginIsolated, 'Is main frame crossOriginIsolated?');

    const reply_token = token();
    const iframe_token = token();

    const iframe = document.createElement('iframe');
    iframe.src = origin + executor_path + coep_require_corp_header +
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
        await receive(reply_token), `${expectedCrossOriginIsolated}`,
        'Is popup crossOriginIsolated?');

    // Test whether the popup's subframe is crossOriginIsolated
    const popup_iframe_token = token();
    const popup_iframe_src = origin + executor_path + coep_require_corp_header +
        corp_cross_origin_header + `&uuid=${popup_iframe_token}`;
    send(iframe_token, `
        const iframe = window.popup.document.createElement('iframe');
        iframe.src = '${popup_iframe_src}';
        popup.document.body.appendChild(iframe);
    `);

    send(
        popup_iframe_token,
        `send('${reply_token}', 'Iframe in popup loaded');`);
    assert_equals(await receive(reply_token), 'Iframe in popup loaded');

    send(
        popup_iframe_token,
        `send('${reply_token}', crossOriginIsolated);`);
    assert_equals(
        await receive(reply_token), `${expectedCrossOriginIsolated}`,
        'Is iframe in popup crossOriginIsolated?');

    // Navigate the popup out of the initial empty document, with COOP:RP and
    // COEP: require-corp. Expect to be crossOriginIsolated.
    const popup_token = token();
    const popup_src = origin + executor_path + coop_restrict_properties_header +
        coep_require_corp_header + `&uuid=${popup_token}`;
    send(iframe_token, `popup.window.location = '${popup_src}';`);

    send(popup_token, `send('${reply_token}', 'Popup loaded');`);
    assert_equals(await receive(reply_token), 'Popup loaded');

    send(popup_token, `send('${reply_token}', crossOriginIsolated);`);
    assert_equals(
        await receive(reply_token), 'true', 'Is popup crossOriginIsolated?');
  }, description);
}

iframePopupAboutBlankTest(
    cross_origin, {expectedCrossOriginIsolated: false}, 'Cross-origin iframe');
iframePopupAboutBlankTest(
    same_origin, {expectedCrossOriginIsolated: true}, 'Same-origin iframe');
