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

  function testRulesetTag(tag, expectedTag, description) {
    promise_test(async t => {
        const agent = await spawnWindow(t);
        const nextUrl = agent.getExecutorURL({ page: 2 });
        await agent.forceSpeculationRules({
            tag,
            prefetch: [{source: "list", urls: [nextUrl]}]
        });
        await agent.navigate(nextUrl);

        const headers = await agent.getRequestHeaders();
        assert_prefetched(headers, "must be prefetched");
        assert_equals(headers.sec_speculation_tags, expectedTag, "Sec-Speculation-Tags");
    }, "Sec-Speculation-Tags [ruleset-based]: " + description);
  }

  function testInvalidRulesetTag(tag, description) {
    testRulesetTag(tag, 'null', description);
  }

  function testRuleTag(tag, expectedTag, description) {
    promise_test(async t => {
        const agent = await spawnWindow(t);
        const nextUrl = agent.getExecutorURL({ page: 2 });
        await agent.forceSpeculationRules({
            prefetch: [{tag, source: "list", urls: [nextUrl]}]
        });
        await agent.navigate(nextUrl);

        const headers = await agent.getRequestHeaders();
        assert_prefetched(headers, "must be prefetched");
        assert_equals(headers.sec_speculation_tags, expectedTag, "Sec-Speculation-Tags");
    }, "Sec-Speculation-Tags [rule-based]: " + description);
  }

  function testInvalidRuleTag(tag, description) {
    promise_test(async t => {
        const agent = await spawnWindow(t);
        const nextUrl = agent.getExecutorURL({ page: 2 });
        await agent.forceSpeculationRules({
            prefetch: [{tag, source: "list", urls: [nextUrl]}]
        });
        await agent.navigate(nextUrl);

        const headers = await agent.getRequestHeaders();
        assert_not_prefetched(headers, "must not be prefetched");
        assert_equals(headers.sec_speculation_tags, "", "Sec-Speculation-Tags");
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
      testInvalidRulesetTag(tag, description);
    } else {
      testInvalidRuleTag(tag, description);
    }
  };
}
