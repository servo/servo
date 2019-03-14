// The list of all feature policies including the sandbox policies.
const all_features = document.featurePolicy.allowedFeatures();

// 'popups' is nonsensical in this test and it is not possible to test 'scripts'
// within this test model.
const ignore_features = ["popups", "scripts"];

// TODO(ekaramad): Figure out different inheritance requirements for different
// policies.
// Features which will be tested for propagation to auxiliary contexts.
const features_that_propagate = all_features.filter(
    (feature) => !ignore_features.includes(feature));

var last_feature_message = null;
var on_new_feature_callback = null;
var on_close_window_callback = null;

function add_iframe(options) {
  assert_true("src" in options, "invalid options");
  var iframe = document.createElement("iframe");
  iframe.src = options.src;
  if ("allow" in options)
    iframe.setAttribute("allow", options.allow);
  if ("sandbox" in options)
    iframe.setAttribute("sandbox", options.sandbox);
  return new Promise( (r) => {
    iframe.addEventListener("load", () => r(iframe));
    document.getElementById("iframe-embedder").appendChild(iframe);
  });
}

// Returns a promise which is resolved with the next/already received message
// with feature update for |feature|. The resolved value is the state of the
// feature |feature|.
function feature_update(feature) {
  function reset_for_next_update() {
    return new Promise((r) => {
      const state = last_feature_message.state;
      last_feature_message = null;
      r(state);
    });
  }
  if (last_feature_message && last_feature_message.feature === feature)
    return reset_for_next_update();

  return new Promise((r) => on_new_feature_callback = r)
            .then(() => reset_for_next_update());
}

function close_aux_window(iframe) {
  return new Promise( (r) => {
    on_close_window_callback = r;
    iframe.contentWindow.postMessage({type: "close_window"}, "*");
  });
}

function on_message(e) {
  var msg = e.data;
  assert_true("type" in msg);
  switch (msg.type) {
    case "feature":
      on_feature_msg(msg);
      break;
    case "close_window":
      on_close_window_msg(msg);
      break;
  }
}

function on_feature_msg(msg) {
  assert_true("feature" in msg);
  assert_true("state" in msg);
  last_feature_message = msg
  if (on_new_feature_callback) {
    on_new_feature_callback();
    on_new_feature_callback = null;
  }
}


function on_close_window_msg(msg) {
  if (on_close_window_callback) {
    on_close_window_callback(msg.result);
    on_close_window_callback = null;
  }
}

window.addEventListener("message", on_message);
