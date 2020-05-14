const features = window.location.search.substr(1).split("&");
const states = features
  .map(feature => [feature, document.featurePolicy.allowsFeature(feature)]);

// TODO(ekaramad): We might at some point choose a different propagation
// strategy with rel=noopener. This test should adapt accordingly (perhaps use
// broadcast channels).
window.opener.parent.postMessage(
    {type: "features", states: states}, "*");
