// META: timeout=long
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=beacon-common.sub.js

'use strict';

const {HTTP_REMOTE_ORIGIN} = get_host_info();

for (const type of [STRING, ARRAYBUFFER, FORM, BLOB]) {
  parallelPromiseTest(async (t) => {
    const iframe = document.createElement('iframe');
    document.body.appendChild(iframe);
    t.add_cleanup(() => iframe.remove());

    const payload = makePayload(SMALL, type);
    const id = token();
    const url = `/beacon/resources/beacon.py?cmd=store&id=${id}`;
    assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));

    iframe.src = `${HTTP_REMOTE_ORIGIN}/common/blank.html`;
  }, `The frame navigates away after calling sendBeacon[type = ${type}].`);
}
