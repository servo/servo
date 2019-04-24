function relTester(submitter, channelInput, title) {
  [
    {
      rel: "",
      exposed: "all"
    },
    {
      rel: "noopener",
      exposed: "noopener"
    },
    {
      rel: "noreferrer",
      exposed: "noreferrer"
    },
    {
      rel: "opener",
      exposed: "all"
    },
    {
      rel: "noopener noreferrer",
      exposed: "noreferrer"
    },
    {
      rel: "noreferrer opener",
      exposed: "noreferrer"
    },
    {
      rel: "opener noopener",
      exposed: "noopener"
    }
  ].forEach(relTest => {
    // Use promise_test to submit only after one test concluded
    promise_test(t => {
      return new Promise(resolve => {
        const channelName = Date.now() + relTest.rel,
              channel = new BroadcastChannel(channelName);
        let form = submitter;
        if (submitter.localName !== "form") {
          form = submitter.form;
        }
        form.rel = relTest.rel;
        channelInput.value = channelName;
        if (submitter.localName !== "form") {
          submitter.click();
        } else {
          submitter.submit();
        }
        channel.onmessage = t.step_func(e => {
          if (relTest.exposed === "all" || relTest.exposed === "noopener") {
            assert_equals(e.data.referrer, window.location.href, "referrer");
          } else {
            assert_equals(e.data.referrer, "", "referrer");
          }
          if (relTest.exposed === "all") {
            assert_true(e.data.haveOpener, "opener");
          } else {
            assert_false(e.data.haveOpener, "opener");
          }
          resolve();
        });
        t.add_cleanup(() => channel.postMessage(null));
      });
    }, `<form rel="${relTest.rel}"> with ${title}`);
  });
}
