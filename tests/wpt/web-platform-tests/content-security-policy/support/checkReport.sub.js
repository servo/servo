(function () {

  // Get values from the substitution engine.
  // We can't just pull these from the document context
  // because this script is intended to be transcluded into
  // another document, and we want the GET values used to request it,
  // not the values for the including document

  // XXX these are unencoded, so there's an unavoidable
  // injection vulnerability in constructing this file...
  // need to upgrade the template engine.
  var reportField  = "{{GET[reportField]}}";
  var reportValue  = "{{GET[reportValue]}}";
  var reportExists = "{{GET[reportExists]}}";
  var noCookies = "{{GET[noCookies]}}";
  var reportCookieName = "{{GET[reportCookieName]}}"
  var testName = "{{GET[testName]}}"
  var cookiePresent = "{{GET[cookiePresent]}}"
  var reportCount = "{{GET[reportCount]}}"

  var location = window.location;
  if (reportCookieName == "") {
    // fallback on test file name if cookie name not specified
    reportCookieName = location.pathname.split('/')[location.pathname.split('/').length - 1].split('.')[0];
  }

  var reportID = "{{GET[reportID]}}";

  if (reportID == "") {
    var cookies = document.cookie.split(';');
    for (var i = 0; i < cookies.length; i++) {
      var cookieName = cookies[i].split('=')[0].trim();
      var cookieValue = cookies[i].split('=')[1].trim();

      if (cookieName == reportCookieName) {
        reportID = cookieValue;
        var cookieToDelete = cookieName + "=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=" + document.location.pathname.substring(0, document.location.pathname.lastIndexOf('/') + 1);
        document.cookie = cookieToDelete;
        break;
      }
    }
  }

  // There is no real way to test (in this particular layer) that a CSP report
  // has *not* been sent, at least not without some major reworks and
  // involvement from all the platform participants. So the current "solution"
  // is to wait for some reasonable amount of time and if no report has been
  // received to conclude that no report has been generated. These timeouts must
  // not exceed the test timeouts set by vendors otherwise the test would fail.
  var timeout = document.querySelector("meta[name=timeout][content=long]") ? 20 : 3;
  var reportLocation = location.protocol + "//" + location.host + "/content-security-policy/support/report.py?op=retrieve_report&timeout=" + timeout + "&reportID=" + reportID;

  if (testName == "") testName = "Violation report status OK.";
  var reportTest = async_test(testName);

  function assert_field_value(field, value, field_name) {
    assert_true(field.indexOf(value.split(" ")[0]) != -1,
                field_name + " value of  \"" + field + "\" did not match " +
                value.split(" ")[0] + ".");
  }

  reportTest.step(function () {

    var report = new XMLHttpRequest();
    report.onload = reportTest.step_func(function () {

        var data = JSON.parse(report.responseText);

        if (data.error) {
          assert_equals("false", reportExists, data.error);
        } else {
          if(reportExists != "" && reportExists == "false" && data["csp-report"]) {
              assert_unreached("CSP report sent, but not expecting one: " + JSON.stringify(data["csp-report"]));
          }
          // Firefox expands 'self' or origins in a policy to the actual origin value
          // so "www.example.com" becomes "http://www.example.com:80".
          // Accomodate this by just testing that the correct directive name
          // is reported, not the details...

          if(data["csp-report"] != undefined && data["csp-report"][reportField] != undefined) {
            assert_field_value(data["csp-report"][reportField], reportValue, reportField);
          } else if (data[0] != undefined && data[0]["body"] != undefined && data[0]["body"][reportField] != undefined) {
            assert_field_value(data[0]["body"][reportField], reportValue, reportField);
          } else {
            assert_equals("", reportField, "Expected report field could not be found in report");
          }
        }

        reportTest.done();
    });

    report.open("GET", reportLocation, true);
    report.send();
  });

  if (noCookies || cookiePresent) {
      var cookieTest = async_test("Test report cookies.");
      var cookieReport = new XMLHttpRequest();
      cookieReport.onload = cookieTest.step_func(function () {
        var data = JSON.parse(cookieReport.responseText);
        if (noCookies) {
          assert_equals(data.reportCookies, "None", "Report should not contain any cookies");
        }

        if (cookiePresent) {
          assert_true(data.reportCookies.hasOwnProperty(cookiePresent), "Report should contain cookie: " + cookiePresent);
        }
        cookieTest.done();
      });
      var cReportLocation = location.protocol + "//" + location.host + "/content-security-policy/support/report.py?op=retrieve_cookies&timeout=" + timeout + "&reportID=" + reportID;
      cookieReport.open("GET", cReportLocation, true);
      cookieReport.send();
  }

  if (reportCount != "") {
      var reportCountTest = async_test("Test number of sent reports.");
      var reportCountReport = new XMLHttpRequest();
      reportCountReport.onload = reportCountTest.step_func(function () {
        var data = JSON.parse(reportCountReport.responseText);

        assert_equals(data.report_count, reportCount, "Report count was not what was expected.");

        reportCountTest.done();
      });
      var cReportLocation = location.protocol + "//" + location.host + "/content-security-policy/support/report.py?op=retrieve_count&timeout=" + timeout + "&reportID=" + reportID;
      reportCountReport.open("GET", cReportLocation, true);
      reportCountReport.send();
  }

})();
