# Controlling Sessions - [Sessions API](../README.md#sessions-api)

It is possible to control the execution of tests on the device under test using the session APIs control methods. They change the status of a session and trigger the device under test to fetch a new url to change location to. Depending on the current status of the session this can be a test or a static page showing information about the current status.

## `start`
The `start` method changes the status of a session from either `PENDING` or `PAUSED` to `RUNNING` and triggers the device under test to execute tests when resuming a paused session.

### HTTP Request

`POST /api/sessions/<session_token>/start`

## `pause`
The `pause` method changes the status of a session from `RUNNING` to `PAUSED` and pauses the execution of tests on the device under test.

### HTTP Request

`POST /api/sessions/<session_token>/pause`

## `stop`
The `stop` method finishes a session early by skipping all pending tests, causing a change of the status to `ABORTED`. It is not possible to undo this action and can only be performed on sessions that are not `ABORTED` or `COMPLETED`.

### HTTP Request

`POST /api/sessions/<session_token>/stop`

