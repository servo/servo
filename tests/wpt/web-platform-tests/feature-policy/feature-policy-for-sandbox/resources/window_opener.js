var auxiliary_context = null;
window.addEventListener("message", (e) => {
  var msg = e.data;
  switch (msg.type) {
    case "features":
      e.source.postMessage({
        type: "features",
        states:
          msg.features
          .map(feature => [feature, document.featurePolicy.allowsFeature(feature)])
      }, "*");
      break;
    case "open_window":
      auxiliary_context = window.open(msg.url);
      break;
    case "close_window":
      if (auxiliary_context)
        auxiliary_context.close();
      e.source.postMessage(
          {type: "close_window", result: auxiliary_context != null}, "*");
  }
});