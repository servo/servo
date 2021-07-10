// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=beacon-common.sub.js

'use strict';

const {ORIGIN} = get_host_info();

// Execute each sample test per redirect status code.
// Note that status codes 307 and 308 are the only codes that will maintain POST
// data through a redirect.
for (const status of [307, 308]) {
  for (const type of [STRING, ARRAYBUFFER, FORM, BLOB]) {
    parallelPromiseTest(async (t) => {
      const iframe = document.createElement('iframe');
      document.body.appendChild(iframe);
      t.add_cleanup(() => iframe.remove());

      const payload = makePayload(SMALL, type);
      const id = token();
      const destination =
          `${ORIGIN}/beacon/resources/beacon.py?cmd=store&id=${id}`;
      const url = `${ORIGIN}/common/redirect.py` +
          `?status=${status}&location=${encodeURIComponent(destination)}`;

      assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
      iframe.remove();

      await waitForResult(id);
    }, `cross-origin, CORS-safelisted: status = ${status}, type = ${type}`);
  }
};

done();
