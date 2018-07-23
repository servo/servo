// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/media-source/

'use strict';

var mediaSource;
var sourceBuffer;
var video = document.createElement("video");

promise_test(async t => {
  const srcs = ['media-source', 'dom', 'html', 'url'];
  const [idl, dom, html, url] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  var idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_dependency_idls(dom);
  idlArray.add_dependency_idls(html);
  idlArray.add_dependency_idls(url);

  const testIdls = new Promise(resolve => {
    try {
      mediaSource = new MediaSource();
      video.src = URL.createObjectURL(mediaSource);
      mediaSource.addEventListener("sourceopen", function () {
        var defaultType ='video/webm;codecs="vp8,vorbis"';
        if (MediaSource.isTypeSupported(defaultType)) {
          sourceBuffer = mediaSource.addSourceBuffer(defaultType);
        } else {
          sourceBuffer = mediaSource.addSourceBuffer('video/mp4');
        }
        sourceBuffer.addEventListener("updateend", function (e) {
          mediaSource.endOfStream();
          resolve();
        });
        sourceBuffer.appendBuffer(new ArrayBuffer());
      });
    } catch (e) {
      // Will be surfaced in idlharness.js's test_object below.
    }
  })

  idlArray.add_objects({
    MediaSource: ['mediaSource'],
    SourceBuffer: ['sourceBuffer'],
    SourceBufferList: ['mediaSource.sourceBuffers']
  });

  const timeout = new Promise((_, reject) => t.step_timeout(reject, 3000));
  return Promise
      .race([testIdls, timeout])
      .then(() => { idlArray.test(); })
      .catch(() => {
        idlArray.test();
        return Promise.reject('Failed to create media-source objects')
      });
}, 'media-source interfaces');
