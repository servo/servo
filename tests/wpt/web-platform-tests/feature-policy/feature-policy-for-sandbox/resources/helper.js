// The list of all feature policies including the sandbox policies.
const all_features = document.featurePolicy.allowedFeatures();

// 'popups' is nonsensical in this test and it is not possible to test 'scripts'
// within this test model.
const ignore_features_for_auxilary_context = ["popups", "scripts"];

// Feature-policies that represent specific sandbox flags.
const sandbox_features = [
    "downloads", "forms", "modals", "orientation-lock",
    "pointer-lock", "popups", "presentation", "scripts", "top-navigation"];

// TODO(ekaramad): Figure out different inheritance requirements for different
// policies.
// Features which will be tested for propagation to auxiliary contexts.
const features_that_propagate = all_features.filter(
    (feature) => !ignore_features_for_auxilary_context.includes(feature));

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

// Resolves after |c| animation frames.
function wait_for_raf_count(c) {
  let count = c;
  let callback = null;
  function on_raf() {
    if (--count === 0) {
      callback();
      return;
    }
    window.requestAnimationFrame(on_raf);
  }
  return new Promise( r => {
    callback = r;
    window.requestAnimationFrame(on_raf);
  });
}

// Returns a promise which is resolved with the next/already received message
// with feature update for |feature|. The resolved value is the state of the
// feature |feature|. If |optional_timeout| is provided, after the given delay
// (in terms of rafs) the promise is resolved with false.
function feature_update(feature, optional_timeout_rafs) {
  function reset_for_next_update() {
    return new Promise((r) => {
      const state = last_feature_message.state;
      last_feature_message = null;
      r(state);
    });
  }
  if (last_feature_message && last_feature_message.feature === feature)
    return reset_for_next_update();

  if (optional_timeout_rafs) {
    wait_for_raf_count(optional_timeout_rafs).then (() => {
      last_feature_message = {state: false};
      on_new_feature_callback();
    });
  }

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
