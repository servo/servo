const windowProps = ["innerWidth", "innerHeight"];

["noopener", "noreferrer"].forEach(openerStyle => {
  async_test(t => {
    const channelName = "34342" + openerStyle + "8907";
    const channel = new BroadcastChannel(channelName);
    window.open("support/sizing-target.html?" + channelName, "", openerStyle);
    channel.onmessage = t.step_func_done(e => {
      // Send message first so if asserts throw the popup is still closed
      channel.postMessage(null);

      for(const prop of windowProps) {
        assert_equals(window[prop], e.data[prop]);
      }
    });
  }, `window.open() with ${openerStyle} should have equal viewport width and height`);
});
