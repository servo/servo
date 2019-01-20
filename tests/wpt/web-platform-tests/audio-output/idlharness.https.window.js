// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediacapture-output/

'use strict';

promise_test(async () => {
  const srcs = ['audio-output', 'dom', 'html'];
  const [idl, dom, html] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  self.audio = document.createElement('audio');
  self.video = document.createElement('video');
  idl_array.add_objects({
    HTMLAudioElement: ['audio'],
    HTMLVideoElement: ['video']
  });
  idl_array.test();
}, 'Test IDL implementation of audio-output API');
