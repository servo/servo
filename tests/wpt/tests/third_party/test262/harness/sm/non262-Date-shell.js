/*---
defines: [runDSTOffsetCachingTestsFraction]
---*/

function runDSTOffsetCachingTestsFraction(part, parts)
{
  var BUGNUMBER = 563938;
  var summary = 'Cache DST offsets to improve SunSpider score';

  var MAX_UNIX_TIMET = 2145859200; // "2037-12-31T08:00:00.000Z" (PST8PDT based!)
  var RANGE_EXPANSION_AMOUNT = 30 * 24 * 60 * 60;

  /**
   * Computes the time zone offset in minutes at the given timestamp.
   */
  function tzOffsetFromUnixTimestamp(timestamp)
  {
    var d = new Date(NaN);
    d.setTime(timestamp); // local slot = NaN, UTC slot = timestamp
    return d.getTimezoneOffset(); // get UTC, calculate local => diff in minutes
  }

  /**
   * Clear the DST offset cache, leaving it initialized to include a timestamp
   * completely unlike the provided one (i.e. one very, very far away in time
   * from it).  Thus an immediately following lookup for the provided timestamp
   * will cache-miss and compute a clean value.
   */
  function clearDSTOffsetCache(undesiredTimestamp)
  {
    var opposite = (undesiredTimestamp + MAX_UNIX_TIMET / 2) % MAX_UNIX_TIMET;

    // Generic purge to known, but not necessarily desired, state
    tzOffsetFromUnixTimestamp(0);
    tzOffsetFromUnixTimestamp(MAX_UNIX_TIMET);

    // Purge to desired state.  Cycle 2x in case opposite or undesiredTimestamp
    // is close to 0 or MAX_UNIX_TIMET.
    tzOffsetFromUnixTimestamp(opposite);
    tzOffsetFromUnixTimestamp(undesiredTimestamp);
    tzOffsetFromUnixTimestamp(opposite);
    tzOffsetFromUnixTimestamp(undesiredTimestamp);
  }

  function computeCanonicalTZOffset(timestamp)
  {
    clearDSTOffsetCache(timestamp);
    return tzOffsetFromUnixTimestamp(timestamp);
  }

  var TEST_TIMESTAMPS_SECONDS =
    [
     // Special-ish timestamps
     0,
     RANGE_EXPANSION_AMOUNT,
     MAX_UNIX_TIMET,
    ];

  var ONE_DAY = 24 * 60 * 60;
  var EIGHTY_THREE_HOURS = 83 * 60 * 60;
  var NINETY_EIGHT_HOURS = 98 * 60 * 60;
  function nextIncrement(i)
  {
    return i === EIGHTY_THREE_HOURS ? NINETY_EIGHT_HOURS : EIGHTY_THREE_HOURS;
  }

  // Now add a long sequence of non-special timestamps, from a fixed range, that
  // overlaps a DST change by "a bit" on each side.  67 days should be enough
  // displacement that we can occasionally exercise the implementation's
  // thirty-day expansion and the DST-offset-change logic.  Use two different
  // increments just to be safe and catch something a single increment might not.
  var DST_CHANGE_DATE = 1268553600; // March 14, 2010
  for (var t = DST_CHANGE_DATE - 67 * ONE_DAY,
           i = nextIncrement(NINETY_EIGHT_HOURS),
           end = DST_CHANGE_DATE + 67 * ONE_DAY;
       t < end;
       i = nextIncrement(i), t += i)
  {
    TEST_TIMESTAMPS_SECONDS.push(t);
  }

  var TEST_TIMESTAMPS =
    TEST_TIMESTAMPS_SECONDS.map(function(v) { return v * 1000; });

  /**************
   * BEGIN TEST *
   **************/

  // Compute the correct time zone offsets for all timestamps to be tested.
  var CORRECT_TZOFFSETS = TEST_TIMESTAMPS.map(computeCanonicalTZOffset);

  // Intentionally and knowingly invoking every single logic path in the cache
  // isn't easy for a human to get right (and know he's gotten it right), so
  // let's do it the easy way: exhaustively try all possible four-date sequences
  // selecting from our array of possible timestamps.

  var sz = TEST_TIMESTAMPS.length;
  var start = Math.floor((part - 1) / parts * sz);
  var end = Math.floor(part / parts * sz);

  for (var i = start; i < end; i++)
  {
    var t1 = TEST_TIMESTAMPS[i];
    for (var j = 0; j < sz; j++)
    {
      var t2 = TEST_TIMESTAMPS[j];
      for (var k = 0; k < sz; k++)
      {
        var t3 = TEST_TIMESTAMPS[k];
        for (var w = 0; w < sz; w++)
        {
          var t4 = TEST_TIMESTAMPS[w];

          clearDSTOffsetCache(t1);

          var tzo1 = tzOffsetFromUnixTimestamp(t1);
          var tzo2 = tzOffsetFromUnixTimestamp(t2);
          var tzo3 = tzOffsetFromUnixTimestamp(t3);
          var tzo4 = tzOffsetFromUnixTimestamp(t4);

          assert.sameValue(tzo1, CORRECT_TZOFFSETS[i]);
          assert.sameValue(tzo2, CORRECT_TZOFFSETS[j]);
          assert.sameValue(tzo3, CORRECT_TZOFFSETS[k]);
          assert.sameValue(tzo4, CORRECT_TZOFFSETS[w]);
        }
      }
    }
  }
}
