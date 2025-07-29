// Utilities for speculation tags tests.

{
  // Retrieves a tag level variant from URLSearchParams of the current window and
  // returns it. Throw an Error if it doesn't have the valid tag level param.
  function getTagLevel() {
    const params = new URLSearchParams(window.location.search);
    const level = params.get('tag-level');
    if (level === null)
      throw new Error('window.location does not have a tag-level param');
    if (level !== 'ruleset' && level !== 'rule')
      throw new Error('window.location does not have a valid tag-level param');
    return level;
  }

  // Retrieves a preloading type variant from URLSearchParams of the current
  // window and returns it. Throw an Error if it doesn't have the valid
  // preloading type param.
  function getPreloadingType() {
    const params = new URLSearchParams(window.location.search);
    const type = params.get('type');
    if (type === null)
      throw new Error('window.location does not have a preloading type param');
    if (type !== 'prefetch' && type !== 'prerender')
      throw new Error('window.location does not have a valid preloading type param');
    return type;
  }

  function assertHeaders(headers, expectedTag, preloadingType) {
    if (expectedTag === undefined) {
      // If `tag` is invalid, preloading should not be
      // triggered, and the navigation should fall back to network. Confirm
      // this behavior by checking the request headers.
      assert_false(headers.has("sec-purpose"));
      assert_false(headers.has("sec-speculation-tags"));
    } else {
      // Make sure the page is preloaded.
      assert_equals(
        headers.get("sec-purpose"),
        preloadingType === "prefetch" ? "prefetch" : "prefetch;prerender");
      assert_equals(headers.get("sec-speculation-tags"), expectedTag);
    }
  }

  function testRulesetTag(tag, expectedTag, description) {
    promise_test(async t => {
        const rcHelper = new RemoteContextHelper();
        const referrerRC = await rcHelper.addWindow(undefined, { features: 'noopener' });

        const extraConfig = {};
        const preloadingType = getPreloadingType();
        const preloadedRC = await referrerRC.helper.createContext({
            executorCreator(url) {
              return referrerRC.executeScript((preloadingType, tag, url, expectedTag) => {
                  const script = document.createElement("script");
                  script.type = "speculationrules";
                  script.textContent = JSON.stringify({
                      tag,
                      [preloadingType]: [
                        {
                          source: "list",
                          urls: [url]
                        }
                      ]
                  });

                  if (expectedTag === undefined) {
                    return new Promise(resolve => {
                      script.addEventListener('error', resolve, { once: true });
                      document.head.append(script);
                    });
                  }

                  document.head.append(script);
              }, [preloadingType, tag, url, expectedTag]);
            }, extraConfig
        });

        // Navigate to the preloaded page.
        referrerRC.navigateTo(preloadedRC.url);

        const headers = await preloadedRC.getRequestHeaders();
        assertHeaders(headers, expectedTag, preloadingType);
    }, "Sec-Speculation-Tags [ruleset-based]: " + description);
  }

  function testRuleTag(tag, expectedTag, description) {
    promise_test(async t => {
        const rcHelper = new RemoteContextHelper();
        const referrerRC = await rcHelper.addWindow(undefined, { features: 'noopener' });

        const extraConfig = {};
        const preloadingType = getPreloadingType();
        const preloadedRC = await referrerRC.helper.createContext({
            executorCreator(url) {
              return referrerRC.executeScript((preloadingType, tag, url) => {
                  const script = document.createElement("script");
                  script.type = "speculationrules";
                  script.textContent = JSON.stringify({
                      [preloadingType]: [
                        {
                          tag,
                          source: "list",
                          urls: [url]
                        }
                      ]
                  });
                  document.head.append(script);
              }, [preloadingType, tag, url]);
            }, extraConfig
        });

        // Navigate to the preloaded page.
        referrerRC.navigateTo(preloadedRC.url);

        const headers = await preloadedRC.getRequestHeaders();
        assertHeaders(headers, expectedTag, preloadingType);
    }, "Sec-Speculation-Tags [rule-based]: " + description);
  }

  // Runs the test function for valid tag cases based on the tag level.
  globalThis.testTag = (tag, expectedTag, description) => {
    if (getTagLevel() === 'ruleset') {
      testRulesetTag(tag, expectedTag, description);
    } else {
      testRuleTag(tag, expectedTag, description);
    }
  };

  // Runs the test function for invalid tag cases based on the tag level.
  globalThis.testInvalidTag = (tag, description) => {
    if (getTagLevel() === 'ruleset') {
      testRulesetTag(tag, undefined, description);
    } else {
      // Pass `undefined` to indicate this preloading is expected to fail.
      testRuleTag(tag, undefined, description);
    }
  };
}
