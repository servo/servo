// The test functions called in the navigation-counter test. They rely on
// artifacts defined in
// '/html/browsers/browsing-the-web/back-forward-cache/resources/helper.sub.js'
// which should be included before this file to use these functions.

function runNavigationIdTest(params, description) {
  const defaultParams = {
    constants: {
      performanceMarkName: 'mark_navigation_id',
      performanceMeasureName: 'measure_navigation_id',
    },
    // This function is to make and obtain the navigation counter value for a
    // performance entries of mark and measure type. It is to be extended for
    // other types of performance entry in future.
    funcBeforeNavigation: (constants) => {
      window.performance.mark(constants.performanceMarkName);
      return window.performance
        .getEntriesByName(constants.performanceMarkName)[0]
        .navigationId;
    },
    funcAfterBFCacheLoad: (expectedNavigationId, constants) => {
      window.performance.mark(
        constants.performanceMarkName + expectedNavigationId);
      window.performance.measure(
        constants.performanceMeasureName + expectedNavigationId,
        constants.performanceMarkName,
        constants.performanceMarkName + expectedNavigationId);
      return [
        window.performance
          .getEntriesByName(
            constants.performanceMarkName + expectedNavigationId)[0]
          .navigationId,
        window.performance
          .getEntriesByName(
            constants.performanceMeasureName + expectedNavigationId)[0]
          .navigationId
      ];
    },
  };
  params = { ...defaultParams, ...params };
  runBfcacheWithMultipleNavigationTest(params, description);
}

function runBfcacheWithMultipleNavigationTest(params, description) {
  const defaultParams = {
    openFunc: url => window.open(url, '_blank', 'noopener'),
    scripts: [],
    funcBeforeNavigation: () => { },
    targetOrigin: originCrossSite,
    navigationTimes: 1,
    funcAfterAssertion: () => { },
  }  // Apply defaults.
  params = { ...defaultParams, ...params };

  promise_test(async t => {
    const pageA = new RemoteContext(token());
    const pageB = new RemoteContext(token());

    const urlA = executorPath + pageA.context_id;
    const urlB = params.targetOrigin + executorPath + pageB.context_id;

    params.openFunc(urlA);

    await pageA.execute_script(waitForPageShow);

    // Assert navigation id is 1 when the document is loaded first time.
    let navigationId = await pageA.execute_script(
      params.funcBeforeNavigation, [params.constants])
    assert_implements_optional(
      navigationId === 1, 'Navigation Id should be 0.');

    for (i = 1; i <= params.navigationTimes; i++) {
      await navigateAndThenBack(pageA, pageB, urlB);

      let navigationIds = await pageA.execute_script(
        params.funcAfterBFCacheLoad, [i + 1, params.constants]);
      assert_implements_optional(
        navigationIds.every(t => t === (i + 1)),
        'Navigation Id should all be ' + (i + 1) + '.');
    }
  }, description);
}
