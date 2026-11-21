// META: title=Cross-origin upload listener registration before and after send()
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

// https://xhr.spec.whatwg.org/#the-send()-method
// https://xhr.spec.whatwg.org/#request-error-steps
// https://fetch.spec.whatwg.org/#cors-preflight-fetch

const body = 'Cross-origin upload';
const uploadEventTypes =
    ['loadstart', 'progress', 'abort', 'error', 'load', 'timeout', 'loadend'];

for (const cors of ['allow', 'deny']) {
  for (const timing of ['late', 'late with upload object', 'early']) {
    promise_test(async t => {
      const url = new URL('/xhr/resources/cors-upload-listener-timing.py',
                          get_host_info().HTTP_REMOTE_ORIGIN);
      url.searchParams.set('id', token());
      url.searchParams.set('cors', cors);
      const reportURL = new URL(url);
      reportURL.searchParams.set('read', '1');

      const xhr = new XMLHttpRequest();
      const events = [];
      const uploadEvents = [];
      const uploadProgress = [];
      let downloadProgress = 0;
      let upload;
      t.add_cleanup(async () => {
        xhr.abort();
        await fetch(reportURL);
      });
      xhr.open('POST', url);
      xhr.setRequestHeader('Content-Type', 'text/plain');
      xhr.addEventListener('readystatechange', () => {
        if (xhr.readyState === XMLHttpRequest.DONE) {
          events.push('xhr.DONE');
        }
      });
      for (const type of ['loadstart', 'abort', 'error', 'load', 'timeout',
                          'loadend']) {
        xhr.addEventListener(type, () => events.push(`xhr.${type}`));
      }
      xhr.addEventListener('progress', () => ++downloadProgress);
      const complete = new Promise(resolve => {
        xhr.addEventListener('loadend', resolve, {once: true});
      });
      const recordUpload = event => {
        uploadEvents.push(event.type);
        events.push(`upload.${event.type}`);
        if (event.type === 'progress') {
          uploadProgress.push(event);
        }
      };

      if (timing !== 'late') {
        upload = xhr.upload;
      }
      if (timing === 'early') {
        upload.addEventListener('loadstart', recordUpload);
      }

      // send() snapshots listener presence, not access to the upload object.
      xhr.send(body);
      upload = xhr.upload;
      for (const type of uploadEventTypes) {
        if (timing !== 'early' || type !== 'loadstart') {
          upload.addEventListener(type, recordUpload);
        }
      }
      await complete;

      assert_equals(xhr.readyState, XMLHttpRequest.DONE);
      if (cors === 'allow') {
        assert_equals(xhr.status, 200);
        assert_equals(xhr.responseText, body);
        assert_greater_than(downloadProgress, 0);
      } else {
        assert_equals(xhr.status, 0);
        assert_equals(xhr.responseText, '');
        assert_equals(downloadProgress, 0);
      }

      if (timing !== 'early') {
        assert_array_equals(uploadEvents, [], 'No late upload events');
        assert_array_equals(events, [
          'xhr.loadstart', 'xhr.DONE',
          cors === 'allow' ? 'xhr.load' : 'xhr.error', 'xhr.loadend'
        ]);
      } else if (cors === 'allow') {
        // End-of-body always fires progress, regardless of upload duration.
        assert_greater_than(uploadProgress.length, 0);
        const finalProgress = uploadProgress[uploadProgress.length - 1];
        assert_true(finalProgress.lengthComputable);
        assert_equals(finalProgress.loaded, body.length);
        assert_equals(finalProgress.total, body.length);
        assert_array_equals(uploadEvents.filter(type => type !== 'progress'),
                            ['loadstart', 'load', 'loadend']);
        assert_array_equals(
            events.filter(type => type.startsWith('xhr.')),
            ['xhr.loadstart', 'xhr.DONE', 'xhr.load', 'xhr.loadend']);
      } else {
        // Request error steps fire DONE, upload error/loadend, then XHR events.
        assert_array_equals(events, [
          'xhr.loadstart', 'upload.loadstart', 'xhr.DONE', 'upload.error',
          'upload.loadend', 'xhr.error', 'xhr.loadend'
        ]);
      }

      const response = await fetch(reportURL);
      assert_true(response.ok, 'Request log is readable');
      const methods = await response.json();
      assert_array_equals(
          methods,
          timing === 'early' ?
              (cors === 'allow' ? ['OPTIONS', 'POST'] : ['OPTIONS']) :
              ['POST'],
          'Only listeners registered before send() force preflight');
    }, `CORS ${cors}, upload listeners ${timing}`);
  }
}
