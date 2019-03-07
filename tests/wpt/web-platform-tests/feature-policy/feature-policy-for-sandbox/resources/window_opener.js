var auxiliary_context = null;
window.addEventListener("message", (e) => {
  var msg = e.data;
  switch (msg.type) {
    case "feature":
      var state = document.featurePolicy.allowsFeature(msg.feature);
      e.source.postMessage({
          type: "feature",
          feature: msg.feature,
          state: state}, "*");
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