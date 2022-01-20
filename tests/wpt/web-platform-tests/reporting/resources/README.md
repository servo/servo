# Using the common report collector

To send reports to the collector, configure the reporting API to POST reports
to the collector's URL. This can be same- or cross- origin with the reporting
document, as the collector will follow the CORS protocol.

The collector supports both CSP Level 2 (report-uri) reports as well as
Reporting API reports.

A GET request can be used to retrieve stored reports for analysis.

A POST request can be used to clear reports stored in the server.

Sent credentials are stored with the reports, and can be retrieved separately.

CORS Notes:
* Preflight requests originating from www2.web-platform.test will be rejected.
  This allows tests to ensure that cross-origin report uploads are not sent when
  the endpoint does not support CORS.

## Supported GET parameters:
 `op`: For GET requests, a string indicating the operation to perform (see
   below for description of supported operations). Defaults to
  `retrieve_report`.

 `reportID`: A UUID to associate with the reports sent from this document. This
   can be used to distinguish between reports from multiple documents, and to
   provide multiple distinct endpoints for a single document. Either `reportID`
   or `endpoint` must be provided.

 `endpoint`: A string which will be used to generate a UUID to be used as the
   reportID. Either `reportID` or `endpoint` must be provided.

 `timeout`: The amount of time to wait, in seconds, before responding. Defaults
   to 0.5s.

 `min_count`: The minimum number of reports to return with the `retrieve_report`
   operation. If there have been fewer than this many reports received, then an
   empty report list will be returned instead.

 `retain`: If present, reports will remain in the stash after being retrieved.
   By default, reports are cleared once retrieved.

### Operations:
 `retrieve_report`: Returns all reports received so far for this reportID, as a
   JSON-formatted list. If no reports have been received, an empty list will be
   returned.

 `retrieve_cookies`: Returns the cookies sent with the most recent reports for
   this reportID, as a JSON-formatted object.

 `retrieve_count`: Returns the number of POST requests for reports with this
   reportID so far.

## Supported POST JSON payload:

  `op`:  For POST requests, a string indicating the operation to perform (see
    below for description of supported operations).

  `reportIDs`: A list of `reportID`s, each one a UUID associated with reports stored in the server stash.

### Operations
`DELETE`: Clear all reports associated with `reportID` listed in `reportIDs` list.

### Example usage:
```
# Clear reports on the server.
fetch('/reporting/resources/report.py', {
  method: "POST",
  body: JSON.stringify({
    op: "DELETE",
    reportIDs: [...] # a list of reportID
  })
});
```
