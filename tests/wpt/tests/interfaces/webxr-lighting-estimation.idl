// GENERATED CONTENT - DO NOT EDIT
// Content was automatically extracted by Reffy into webref
// (https://github.com/w3c/webref)
// Source: WebXR Lighting Estimation API Level 1 (https://immersive-web.github.io/lighting-estimation/)

[SecureContext, Exposed=Window]
interface XRLightProbe : EventTarget {
  readonly attribute XRSpace probeSpace;
  attribute EventHandler onreflectionchange;
};

enum XRReflectionFormat {
  "srgba8",
  "rgba16f",
};

[SecureContext, Exposed=Window]
interface XRLightEstimate {
  readonly attribute Float32Array sphericalHarmonicsCoefficients;
  readonly attribute DOMPointReadOnly primaryLightDirection;
  readonly attribute DOMPointReadOnly primaryLightIntensity;
};

dictionary XRLightProbeInit {
  XRReflectionFormat reflectionFormat = "srgba8";
};

partial interface XRSession {
  Promise<XRLightProbe> requestLightProbe(optional XRLightProbeInit options = {});
  readonly attribute XRReflectionFormat preferredReflectionFormat;
};

partial interface XRFrame {
  XRLightEstimate? getLightEstimate(XRLightProbe lightProbe);
};

partial interface XRWebGLBinding {
  WebGLTexture? getReflectionCubeMap(XRLightProbe lightProbe);
};
