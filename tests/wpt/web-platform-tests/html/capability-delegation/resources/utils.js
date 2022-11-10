// Returns a Promise that gets resolved with `event.data` when `window` receives from `source` a
// "message" event whose `event.data.type` matches the string `message_data_type`.
function getMessageData(message_data_type, source) {
    return new Promise(resolve => {
        function waitAndRemove(e) {
            if (e.source != source || !e.data || e.data.type != message_data_type)
                return;
            window.removeEventListener("message", waitAndRemove);
            resolve(e.data);
        }
        window.addEventListener("message", waitAndRemove);
    });
}

// A helper that simulates user activation on the current frame if `activate` is true, then posts
// `message` to `frame` with the target `origin` and specified `capability` to delegate. This helper
// awaits and returns a Promise fulfilled with the result message sent in reply from `frame`.
// However, if the `postMessage` call fails, the helper returns a Promise rejected with the
// exception.
async function postCapabilityDelegationMessage(frame, message, origin, capability, activate) {
    let result_promise = getMessageData("result", frame);

    if (activate)
        await test_driver.bless();

    let postMessageOptions = {targetOrigin: origin};
    if (capability)
        postMessageOptions["delegate"] = capability;
    try {
        frame.postMessage(message, postMessageOptions);
    } catch (exception) {
        return Promise.reject(exception);
    }

    return await result_promise;
}

// Returns the name of a capability for which `postMessage` delegation is supported by the user
// agent, or undefined if no such capability is found.
async function findOneCapabilitySupportingDelegation() {
  const capabilities = ["fullscreen", "payment"];

  for (let i = 0; i < capabilities.length; i++) {
    try {
      await postCapabilityDelegationMessage(window, "any_message", "/", capabilities[i], false);
      assert_unreached();
    } catch (exception) {
      if (exception.name != "NotSupportedError")
          return capabilities[i];
      // Ignore all other exceptions to continue searching through the list.
    }
  };

  return undefined;
}
