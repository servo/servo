// Compares a performance entry to a predefined one
// perfEntriesToCheck is an array of performance entries from the user agent
// expectedEntries is an array of performance entries minted by the test
function checkEntries(perfEntriesToCheck, expectedEntries) {
  function findMatch(pe) {
    // we match based on entryType and name
    for (var i = expectedEntries.length - 1; i >= 0; i--) {
      var ex = expectedEntries[i];
      if (ex.entryType === pe.entryType && ex.name === pe.name) {
        return ex;
      }
    }
    return null;
  }

  assert_equals(perfEntriesToCheck.length, expectedEntries.length, "performance entries must match");

  perfEntriesToCheck.forEach(function (pe1) {
    assert_not_equals(findMatch(pe1), null, "Entry matches");
  });
}
