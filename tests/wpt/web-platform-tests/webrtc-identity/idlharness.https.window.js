// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['webrtc-identity'],
  ['webrtc', 'mediacapture-streams', 'html', 'dom', 'WebIDL'],
  async idlArray => {
    idlArray.add_objects({
      RTCPeerConnection: [`new RTCPeerConnection()`],
      RTCIdentityAssertion: [`new RTCIdentityAssertion('idp', 'name')`],
      MediaStreamTrack: ['track'],
      // TODO: RTCIdentityProviderGlobalScope
      // TODO: RTCIdentityProviderRegistrar
    });

    try {
      self.track = await navigator.mediaDevices
          .getUserMedia({audio: true})
          .then(m => m.getTracks()[0]);
    } catch (e) {}
  }
);
