var callback = arguments[arguments.length - 1];
window.opener.testdriver_callback = function(results) {
  /**
   * The current window and its opener belong to the same domain, making it
   * technically possible for data structures to be shared directly.
   * Unfortunately, some browser/WebDriver implementations incorrectly
   * serialize Arrays from foreign realms [1]. This issue does not extend to
   * the behavior of `JSON.stringify` and `JSON.parse` in these
   * implementations. Use that API to re-create the data structure in the local
   * realm to avoid the problem in the non-conforming browsers.
   *
   * [1] This has been observed in Edge version 17 and/or the corresponding
   *     release of Edgedriver
   */
  try {
    results = JSON.parse(JSON.stringify(results));
  } catch (error) {}

  callback(results);
};
window.opener.process_next_event();
