// META: title=IndexedDB: large nested objects are cloned correctly
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/nested-cloning-common.js
// META: timeout=long

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

cloningTestWithKeyGenerator(
    'multiple requests of objects with blobs and large typed arrays', [
      {
        blob: {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink1',
          seed: 1
        },
        more: [
          {type: 'buffer', size: wrapThreshold, seed: 2},
          {
            type: 'blob',
            size: wrapThreshold,
            mimeType: 'text/x-blink3',
            seed: 3
          },
          {type: 'buffer', size: wrapThreshold, seed: 4},
        ],
        blob2: {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink5',
          seed: 5
        },
      },
      [
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink06',
          seed: 6
        },
        {type: 'buffer', size: wrapThreshold, seed: 7},
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink08',
          seed: 8
        },
        {type: 'buffer', size: wrapThreshold, seed: 9},
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink10',
          seed: 10
        },
      ],
      {
        data: [
          {
            type: 'blob',
            size: wrapThreshold,
            mimeType: 'text/x-blink-11',
            seed: 11
          },
          {type: 'buffer', size: wrapThreshold, seed: 12},
          {
            type: 'blob',
            size: wrapThreshold,
            mimeType: 'text/x-blink-13',
            seed: 13
          },
          {type: 'buffer', size: wrapThreshold, seed: 14},
          {
            type: 'blob',
            size: wrapThreshold,
            mimeType: 'text/x-blink-15',
            seed: 15
          },
        ],
      },
      [
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink16',
          seed: 16
        },
        {type: 'buffer', size: wrapThreshold, seed: 17},
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink18',
          seed: 18
        },
        {type: 'buffer', size: wrapThreshold, seed: 19},
        {
          type: 'blob',
          size: wrapThreshold,
          mimeType: 'text/x-blink20',
          seed: 20
        },
      ],
    ]);
