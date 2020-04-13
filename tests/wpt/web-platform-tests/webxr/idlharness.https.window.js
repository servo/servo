// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://immersive-web.github.io/webxr/

idl_test(
  ['webxr'],
  ['permissions', 'webgl1', 'geometry', 'html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      XR: ['navigator.xr'],
      // TODO: XRSystem
      XRSession: ['xrSession'],
      XRRenderState: ['xrRenderState'],
      // TODO: XRFrame
      // TODO: XRSpace
      XRReferenceSpace: ['xrReferenceSpace'],
      // TODO: XRBoundedReferenceSpace
      // TODO: XRView
      // TODO: XRViewport
      XRRigidTransform: ['new XRRigidTransform()'],
      // TODO: XRPose
      // TODO: XRViewerPose
      // TODO: XRInputSource
      XRInputSourceArray: ['xrInputSourceArray'],
      XRWebGLLayer: ['xrWebGLLayer'],
      WebGLRenderingContextBase: ['webGLRenderingContextBase'],
      XRSessionEvent: ['xrSessionEvent'],
      // TODO: XRInputSourceEvent
      XRInputSourcesChangeEvent: ['xrInputSourcesChangeEvent'],
      // TODO: XRReferenceSpaceEvent
      // TODO: XRPermissionStatus
    });

    self.xrSession = await navigator.xr.requestSession('inline');
    self.xrRenderState = self.xrSession.renderState;
    self.xrReferenceSpace = await self.xrSession.requestReferenceSpace('viewer');
    self.xrInputSourceArray = self.xrSession.inputSources;
    self.xrSessionEvent = new XRSessionEvent('end', {session: self.xrSession});
    self.xrInputSourcesChangeEvent = new XRInputSourcesChangeEvent('inputsourceschange', {
      session: self.xrSession,
      added: [],
      removed: [],
    });

    // XRWebGLRenderingContext is a typedef to either WebGLRenderingContext or WebGL2RenderingContext.
    const canvas = document.createElement('canvas');
    self.webGLRenderingContextBase = canvas.getContext('webgl');
    self.xrWebGLLayer = new XRWebGLLayer(self.xrSession, self.webGLRenderingContextBase);
  }
);
