// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js

'use strict';

const bodyText = 'Background Fetch request body';
const encodedBody = () => new TextEncoder().encode(bodyText);

const bodyTests = [
  {
    name: 'string',
    create: () => bodyText,
  },
  {
    name: 'ArrayBuffer',
    create: () => encodedBody().buffer,
  },
  {
    name: 'TypedArray',
    create: () => encodedBody(),
  },
  {
    name: 'DataView',
    create: () => {
      const body = encodedBody();
      return new DataView(body.buffer, body.byteOffset, body.byteLength);
    },
  },
  {
    name: 'Blob',
    create: () => new Blob([bodyText]),
  },
  {
    name: 'File',
    create: () => new File([bodyText], 'body.txt'),
  },
  {
    name: 'URLSearchParams',
    create: () => new URLSearchParams({body: bodyText}),
  },
  {
    name: 'FormData',
    create: () => {
      const body = new FormData();
      body.append('body', bodyText);
      return body;
    },
  },
];

for (const bodyTest of bodyTests) {
  backgroundFetchTest(async (test, backgroundFetch) => {
    const request = new Request('resources/upload.py', {
      method: 'POST',
      body: bodyTest.create(),
      ...bodyTest.requestOptions,
    });
    const expectedBody = new Uint8Array(await request.clone().arrayBuffer());
    const registration =
        await backgroundFetch.fetch(uniqueId(), [request, '/common/slow.py']);
    test.add_cleanup(() => registration.abort());

    const record =
        await registration.match('resources/upload.py', {ignoreMethod: true});
    assert_not_equals(record, undefined);
    assert_equals(record.request.method, 'POST');
    assert_array_equals(new Uint8Array(await record.request.arrayBuffer()),
                        expectedBody);
  }, `Match returns the request body for ${bodyTest.name}.`);
}
