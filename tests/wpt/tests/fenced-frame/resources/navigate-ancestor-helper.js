async function runNavigateAncestorTest(test_type, ancestor_type) {
  // Set up a detector to check that the top-level page doesn't navigate away.
  window.onbeforeunload =
      e => {
        assert_unreached(
            `The top-level test runner document does not navigate when a ` +
            `${test_type} navigates ${ancestor_type}`);
      }

  let fenced_frame = await attachFencedFrameContext();
  await multiClick(10, 10, fenced_frame.element);

  // This is the page that the inner frames will navigate to.
  const [uuid, url] = generateRemoteContextURL([]);

  switch (test_type) {
    case 'top-level fenced frame':
      // This fenced frame will attempt to navigate its parent. It should end up
      // navigating *itself* since it is a top-level browsing context. Just in
      // case it accidentally navigates *this* frame, we have an
      // `onbeforeunload` handler that will automatically fail the test before.
      await fenced_frame.execute(async (url, ancestor_type) => {
        window.executor.suspend(() => {
          window[ancestor_type].location = url;
        });
      }, [url, ancestor_type]);
      // Ensure that a navigation took place via the `window.location` call.
      fenced_frame.context_id = uuid;
      await fenced_frame.execute(() => {});
      break;
    case 'nested fenced frame':
      await fenced_frame.execute(async (url, uuid, ancestor_type) => {
        const inner_fenced_frame = await attachFencedFrameContext();
        await inner_fenced_frame.execute((url, ancestor_type) => {
          window.executor.suspend(() => {
            window[ancestor_type].location = url;
          });
        }, [url, ancestor_type]);
        // Ensure that a navigation took place via the `window.location` call.
        inner_fenced_frame.context_id = uuid;
        await inner_fenced_frame.execute(() => {});
      }, [url, uuid, ancestor_type]);
      // Check that the root fenced frame did not unload. The test will time out
      // if it did.
      await fenced_frame.execute(() => {});
      break;
    case 'nested iframe':
      // When the iframe tries to navigate its ancestor frame, it should not
      // navigate *this* frame, because the sandboxed navigation browsing
      // context flag must be set in fenced frame trees. See:
      // https://html.spec.whatwg.org/multipage/origin.html#sandboxed-navigation-browsing-context-flag
      await fenced_frame.execute(async (url, ancestor_type) => {
        const inner_iframe = await attachIFrameContext();
        await inner_iframe.execute((url, ancestor_type) => {
          try {
            window[ancestor_type].location = url;
            assert_unreached(
                'The navigation from the nested iframe should ' +
                'not be successful.');
          } catch (error) {
            assert_equals(error.name, 'SecurityError');
          }
        }, [url, ancestor_type]);
      }, [url, ancestor_type]);
      // Check that the root fenced frame did not unload. The test will time out
      // if it did.
      await fenced_frame.execute(() => {});
      break;
  }
}
