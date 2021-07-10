# REST API - [WAVE Test Suite](../README.md)

The REST API allows the WAVE server to be integrated into other systems. Every 
call must be preceded with a namespace or web root, which is omitted in this 
documentation. The default web root is `/_wave`, which can be changed in the 
config.json using the keyword `web_root`.

## Sessions API <a name="sessions-api"></a>

| Name                                           | Description                                          |
| ---------------------------------------------- | ---------------------------------------------------- |
| [`create`](./sessions-api/create.md)           | Creates a new test session.                          |
| [`read`](./sessions-api/read.md)               | Reads a sessions configuration.                      |
| [`read public`](./sessions-api/read-public.md) | Reads all public sessions tokens.                    |
| [`update`](./sessions-api/update.md)           | Updates a session configuration.                     |
| [`delete`](./sessions-api/delete.md)           | Deletes a test session.                              |
| [`status`](./sessions-api/status.md)           | Reads the status and progress of a session.          |
| [`start`](./sessions-api/control.md#start)     | Starts a test session.                               |
| [`stop`](./sessions-api/control.md#stop)       | Stops a test session.                                |
| [`pause`](./sessions-api/control.md#pause)     | Pauses a test session.                               |
| [`find`](./sessions-api/find.md)               | Finds a session token by providing a token fragment. |
| [`labels`](./sessions-api/labels.md)           | Attach labels to sessions for organization purposes. |
| [`events`](./sessions-api/events.md)           | Register for sessions specific events.               |

## Tests API <a name="tests-api"></a>

| Name                                                            | Description                                            |
| --------------------------------------------------------------- | ------------------------------------------------------ |
| [`read all`](./tests-api/read-all.md)                           | Reads all tests available.                             |
| [`read session`](./tests-api/read-session.md)                   | Reads all tests that are part of a session.            |
| [`read next`](./tests-api/read-next.md)                         | Reads the next test to run in a session.               |
| [`read last completed`](./tests-api/read-last-completed.md)     | Reads the last completed tests of a session.           |
| [`read malfunctioning`](./tests-api/read-malfunctioning.md)     | Reads the list of malfunctioning tests of a session.   |
| [`update malfunctioning`](./tests-api/update-malfunctioning.md) | Updates the list of malfunctioning tests of a session. |
| [`read available apis`](./tests-api/read-available-apis.md)     | Reads all available APIs names and paths.              |

## Results API <a name="results-api"></a>

| Name                                                                         | Description                                                                     |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| [`create`](./results-api/create.md)                                          | Create a new test result for a test in a session.                               |
| [`read`](./results-api/read.md)                                              | Read all test results of a session.                                             |
| [`read compact`](./results-api/read-compact.md)                              | Read the number of passed, failed, timed out and not run tests of a session.    |
| [`config`](./results-api/config.md)                                          | Read what features of the results API are enabled.                              |
| [`import`](./results-api/import.md)                                          | Import session results.                                                         |
| [`import enabled`](./results-api/import.md#2-import-enabled)                 | Check whether or not the import feature is enabled.                             |
| [`download`](./results-api/download.md#1-download)                           | Download all session results to import into other WMATS instance.               |
| [`download api`](./results-api/download.md#2-download-api)                   | Download all results of an API.                                                 |
| [`download all apis`](./results-api/download.md#3-download-all-apis)         | Download all results of all APIs.                                               |
| [`view report`](./results-api/download.md#4-download-report)                 | View the WPT report of an API of a session.                                     |
| [`view multi report`](./results-api/download.md#5-download-multi-report)     | View the WPT report of an API of multiple sessions.                             |
| [`download overview`](./results-api/download.md#6-download-overview)         | Download an overview of results of all APIs of a session.                       |
| [`view report`](./results-api/view.md#1-view-report)                         | Read an url to a hosted version of a WPT report for an API of a session.        |
| [`view multi report`](./results-api/view.md#2-view-multi-report)             | Read an url to a hosted version of a WPT report for an API of multiple session. |
