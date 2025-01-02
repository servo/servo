// Verify that an iframe whose URL "matches about:blank" [1] uses its about base
// URL and initiator/creator origin.
//
// [1]: https://html.spec.whatwg.org/#matches-about:blank
onload = () => {
  async_test(t => {
    const iframe = document.createElement('iframe');
    // Navigate the iframe away from the initial about:blank Document [1] to
    // isolate the effects of a "matches about:blank" URL.
    //
    // [1]: https://html.spec.whatwg.org/#is-initial-about:blank
    iframe.src = '/common/blank.html';

    iframe.addEventListener('load', e => {
      assert_equals(iframe.contentWindow.location.pathname, '/common/blank.html');

      // Navigate the iframe to a URL that "matches about:blank" but isn't exactly
      // "about:blank".
      iframe.onload = t.step_func_done(() => {
        assert_equals(iframe.contentDocument.URL, 'about:blank?query#fragment');
        assert_equals(iframe.contentWindow.origin, window.origin);
        assert_equals(iframe.contentDocument.baseURI, document.baseURI);
      });
      iframe.src = 'about:blank?query#fragment';
    }, {once: true});

    document.body.appendChild(iframe);
  }, "about:blank and about:blank?foo#bar both 'match about:blank'");
};
