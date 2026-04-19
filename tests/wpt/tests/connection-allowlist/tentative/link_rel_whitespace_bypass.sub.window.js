// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// Tests that whitespace characters (tab, newline, carriage return, space)
// preceding or following rel type keywords in <link> elements do not bypass
// Connection-Allowlist enforcement. The HTML spec defines rel attribute values
// as sets of space-separated tokens split on ASCII whitespace, so browsers
// correctly recognize e.g. "\tprefetch" as a prefetch hint. Connection-Allowlist
// enforcement must also handle this normalization.
//
// Regression tests for Connection-Allowlist whitespace bypass.
// See also: link_rel_prefetch.sub.window.js for baseline prefetch tests.

const port = get_host_info().HTTP_PORT_ELIDED;
const BLOCKED_ORIGIN = 'http://{{hosts[][www]}}' + port;

function whitespace_bypass_test(rel_value, display_name) {
  promise_test(async t => {
    const key = token();
    const value = 'leaked';
    const params = new URLSearchParams();
    params.set('key', key);
    params.set('value', value);

    const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

    // Use setAttribute to set the rel value with embedded whitespace.
    // The browser's link processing splits rel on ASCII whitespace and
    // recognizes the token (e.g. "prefetch"). Connection-Allowlist must
    // also handle this correctly and block the request.
    const link = document.createElement('link');
    link.setAttribute('rel', rel_value);
    link.href = url;
    document.head.appendChild(link);
    t.add_cleanup(() => link.remove());

    // If the allowlist check is bypassed, the prefetch request reaches
    // the server and stores the value. Verify it does not.
    const result = await Promise.race([
      new Promise(r => t.step_timeout(r, 2000)),
      nextValueFromServer(key)
    ]);
    assert_equals(result, undefined,
        `Prefetch should be blocked for <link rel="${display_name}">.`);
  }, `<link rel="${display_name}"> to blocked origin must be blocked by Connection-Allowlist.`);
}

// Also test via innerHTML, matching the bypass technique where the HTML
// parser's whitespace normalization may differ from
// Connection-Allowlist's enforcement checking.
function whitespace_bypass_test_innerHTML(rel_value, display_name) {
  promise_test(async t => {
    const key = token();
    const value = 'leaked';
    const params = new URLSearchParams();
    params.set('key', key);
    params.set('value', value);

    const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

    const container = document.createElement('div');
    container.innerHTML = `<link rel="${rel_value}" href="${url}">`;
    const link = container.querySelector('link');
    assert_not_equals(link, null, 'Link element should be created by innerHTML');
    document.head.appendChild(link);
    t.add_cleanup(() => link.remove());

    const result = await Promise.race([
      new Promise(r => t.step_timeout(r, 2000)),
      nextValueFromServer(key)
    ]);
    assert_equals(result, undefined,
        `Prefetch via innerHTML should be blocked for rel="${display_name}".`);
  }, `innerHTML: <link rel="${display_name}"> to blocked origin must be blocked by Connection-Allowlist.`);
}

// --- setAttribute tests ---

// Tab character (U+0009) before rel type.
whitespace_bypass_test('\tprefetch', '\\tprefetch');

// Space character (U+0020) before rel type.
whitespace_bypass_test(' prefetch', '(space)prefetch');

// Line feed (U+000A) before rel type.
whitespace_bypass_test('\nprefetch', '\\nprefetch');

// Carriage return (U+000D) before rel type.
whitespace_bypass_test('\rprefetch', '\\rprefetch');

// Form feed (U+000C) before rel type.
whitespace_bypass_test('\fprefetch', '\\fprefetch');

// Trailing whitespace after rel type.
whitespace_bypass_test('prefetch\t', 'prefetch\\t');

// Multiple whitespace characters surrounding rel type.
whitespace_bypass_test('\t prefetch', '\\t(space)prefetch');
whitespace_bypass_test(' \tprefetch\t ', '(space)\\tprefetch\\t(space)');

// --- innerHTML tests (matching the bypass technique) ---

// Tab character before rel type via innerHTML.
whitespace_bypass_test_innerHTML('\tprefetch', '\\tprefetch');

// Space before rel type via innerHTML.
whitespace_bypass_test_innerHTML(' prefetch', '(space)prefetch');

// --- HTML entity encoding tests ---
// Verify that HTML character references in rel attribute values do not bypass
// Connection-Allowlist enforcement. When parsed by the HTML parser,
// "preco&#110;&#110;ect" decodes to "preconnect".

function entity_encoded_rel_test(innerHTML_fragment, display_name) {
  promise_test(async t => {
    const key = token();
    const value = 'leaked';
    const params = new URLSearchParams();
    params.set('key', key);
    params.set('value', value);

    const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

    // Use innerHTML so the HTML parser decodes character references.
    const container = document.createElement('div');
    container.innerHTML = innerHTML_fragment.replace('HREF_URL', url);
    const link = container.querySelector('link');
    if (link) {
      document.head.appendChild(link);
      t.add_cleanup(() => link.remove());
    }

    const result = await Promise.race([
      new Promise(r => t.step_timeout(r, 2000)),
      nextValueFromServer(key)
    ]);
    assert_equals(result, undefined,
        `Request should be blocked for ${display_name}.`);
  }, `${display_name} to blocked origin must be blocked by Connection-Allowlist.`);
}

// "preco&#110;&#110;ect" decodes to "preconnect" via HTML character references.
entity_encoded_rel_test(
    '<link rel="preco&#110;&#110;ect" href="HREF_URL">',
    'HTML entity encoded rel="preco&#110;&#110;ect" (preconnect)');

// "pre&#102;etch" decodes to "prefetch".
entity_encoded_rel_test(
    '<link rel="pre&#102;etch" href="HREF_URL">',
    'HTML entity encoded rel="pre&#102;etch" (prefetch)');

// "dns-pre&#102;etch" decodes to "dns-prefetch".
entity_encoded_rel_test(
    '<link rel="dns-pre&#102;etch" href="HREF_URL">',
    'HTML entity encoded rel="dns-pre&#102;etch" (dns-prefetch)');
