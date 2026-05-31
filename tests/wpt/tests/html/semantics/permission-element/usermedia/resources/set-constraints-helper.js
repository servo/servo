function runSetConstraintsTests(testCases) {
  for (let i = 0; i < testCases.length; i++) {
    const testCase = testCases[i];
    const { type, video, audio, expVideo, expAudio } = testCase;
    const testName = `Case ${i}: type='${type}', video=${video}, audio=${audio}`;

    promise_test(async (t) => {
      await test_driver.set_permission({ name: "camera" }, "granted");
      await test_driver.set_permission({ name: "microphone" }, "granted");

      const usermedia = document.createElement("usermedia");
      if (type !== null) {
        usermedia.setAttribute("type", type);
      }
      document.body.appendChild(usermedia);
      t.add_cleanup(() => {
        if (usermedia.parentNode) {
          document.body.removeChild(usermedia);
        }
      });

      const stream_promise = new Promise((resolve) => {
        usermedia.onstream = resolve;
      });

      const constraints = {};
      if (video !== undefined) {
        constraints.video = video ? {} : undefined;
      }
      if (audio !== undefined) {
        constraints.audio = audio ? {} : undefined;
      }
      usermedia.setConstraints(constraints);

      // should_trigger is true if the element has at least one permission descriptor.
      const should_trigger = type !== null || video === true || audio === true;

      // Wait until the element is clickable.
      await new Promise((r) => t.step_timeout(r, 600));

      if (should_trigger) {
        await test_driver.click(usermedia);
        await stream_promise;
      } else {
        await test_driver.click(usermedia);
        // Wait a bit to ensure no stream event is fired
        await new Promise((r) => t.step_timeout(r, 200));
      }

      const stream = usermedia.stream;
      t.add_cleanup(() => {
        if (stream) {
          stream.getTracks().forEach((track) => track.stop());
        }
      });

      if (expVideo || expAudio) {
        assert_true(stream instanceof MediaStream, `stream type check`);
        assert_equals(
          stream.getVideoTracks().length,
          expVideo,
          `video tracks mismatch`,
        );
        assert_equals(
          stream.getAudioTracks().length,
          expAudio,
          `audio tracks mismatch`,
        );
      } else {
        assert_equals(stream, null, `stream should be null`);
        if (should_trigger) {
          assert_true(
            usermedia.error !== null && usermedia.error !== undefined,
            `error attribute should be set`,
          );
        }
      }
    }, testName);
  }
}
