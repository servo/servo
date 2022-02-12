function formUsesTargetBlank(submitter) {
  if (submitter.formTarget && submitter.formTarget === "_blank") {
    return true;
  }
  if (submitter.form && submitter.form.target === "_blank") {
    return true;
  }
  if (submitter.target && submitter.target === "_blank") {
    return true;
  }
  if (submitter.getRootNode().querySelector("base").target === "_blank") {
    return true;
  }
  return false;
}

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
          // When rel is not explicitly given, account for target=_blank defaulting to noopener
          if (relTest.exposed === "all" && !(relTest.rel === "" && formUsesTargetBlank(submitter))) {
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
