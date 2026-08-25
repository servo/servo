const DOCUMENT_LASTMODIFIED_REGEX = /^([0-9]{2})\/([0-9]{2})\/([0-9]{4}) ([0-9]{2}):([0-9]{2}):([0-9]{2})$/;

function assert_document_lastmodified_string_approximately_now(str) {
  // We want to test that |str| was a time in the user's local
  // timezone generated within a few seconds prior to the present.
  // This requires some care, since it is possible that:
  //  - the few second difference may have crossed a
  //    year/month/day/hour/minute boundary
  //  - the few second difference may have crossed a change in the
  //    local timezone's UTC offset
  //  - the local time might be one that has multiple valid UTC
  //    representations (for example, because it's in the hour
  //    following a shift from summer time to winter time)
  // We will make some assumptions to do this:
  //  - local time's UTC offset doesn't change more than once per
  //    minute
  //  - local time's UTC offset only changes by integral numbers of
  //    minutes

  // The date must be equal to or earlier than the present time.
  var dmax = new Date();

  // The date must be equal to or later than 2.5 seconds ago.
  var TOLERANCE_MILLISECONDS = 2500;
  var dmin = new Date();
  dmin.setTime(dmax.getTime() - TOLERANCE_MILLISECONDS);

  // Extract the year/month/date/hours/minutes/seconds from str.  It
  // is important that we do *not* try to construct a Date object from
  // these, since the core of the date object is a timestamp in UTC,
  // and there are cases (such as the hour on each side of a change
  // from summer time to winter time) where there are multiple
  // possible UTC timestamps for a given YYYY-MM-DD HH:MM:SS, and
  // constructing a Date object would pick one of them, which might be
  // the wrong one.  However, we already have the right one in dmin
  // and dmax, so we should instead extract local time from those
  // rather than converting these values to UTC.
  var m = DOCUMENT_LASTMODIFIED_REGEX.exec(str);
  var syear = Number(m[3]);
  var smonth = Number(m[1]) - 1; // match Javascript 0-based months
  var sdate = Number(m[2]);
  var shours = Number(m[4]);
  var sminutes = Number(m[5]);
  var sseconds = Number(m[6]);

  if (dmin.getFullYear() == dmax.getFullYear() &&
      dmin.getMonth() == dmax.getMonth() &&
      dmin.getDate() == dmax.getDate() &&
      dmin.getHours() == dmax.getHours() &&
      dmin.getMinutes() == dmax.getMinutes()) {
    // min and max have the same minute
    assert_equals(smonth, dmin.getMonth(), "month");
    assert_equals(sdate, dmin.getDate(), "date");
    assert_equals(syear, dmin.getFullYear(), "year");
    assert_equals(shours, dmin.getHours(), "hours");
    assert_equals(sminutes, dmin.getMinutes(), "minutes");
    assert_true(dmin.getSeconds() <= sseconds &&
                sseconds <= dmax.getSeconds(), "seconds");
  } else if (dmin.getFullYear() == syear &&
             dmin.getMonth() == smonth &&
             dmin.getDate() == sdate &&
             dmin.getHours() == shours &&
             dmin.getMinutes() == sminutes) {
    // actual value has the same minute as min
    assert_true(dmin.getSeconds() <= sseconds, "dmin.getSeconds() <= sseconds");
    assert_true(57 <= dmin.getSeconds(), "unexpected local time rules (dmin match)");
  } else if (dmax.getFullYear() == syear &&
             dmax.getMonth() == smonth &&
             dmax.getDate() == sdate &&
             dmax.getHours() == shours &&
             dmax.getMinutes() == sminutes) {
    // actual value has the same minute as max
    assert_true(sseconds <= dmax.getSeconds(), "sseconds <= dmax.getSeconds()");
    assert_true(dmax.getSeconds() <= 2, "unexpected local time rules (dmax match)");
  } else {
    assert_unreached("unexpected local time rules (no match)");
  }
}
