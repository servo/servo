function testResourceInitiatorUrl(resourceName, expectedUrl) {
    return new Promise(( resolve , reject) => {
      const observer = new PerformanceObserver(list => {
        const entries = list.getEntriesByType('resource');
        for (const entry of entries) {
          if (entry.name.endsWith(resourceName)) {
            observer.disconnect();
            assert_equals(entry.initiatorUrl, expectedUrl, `Test ${resourceName} initiatorUrl`);
            resolve();
            return;
          }
        }
        reject(resourceName + " not found");
      });
      observer.observe({type: "resource", buffered: true});
    });
}

// TODO(guohuideng@microsoft.com): The utility function below is used by some tests designed
// for resource IDs. Either delete them or modify them when the resource-initiator feature
// is complete.
function testResourceInitiator(resourceName, expectedInitiator) {
    return new Promise(resolve => {
      const observer = new PerformanceObserver(list => {
        const entries = list.getEntriesByType('resource');
        for (const entry of entries) {
          if (entry.name.endsWith(resourceName)) {
            observer.disconnect();
            assert_equals(entry.initiator, expectedInitiator, `Test ${resourceName} initiator`);
            resolve();
            return;
          }
        }
      });
      observer.observe({entryTypes: ['resource']});
    });
}
