# `status` - [Sessions API](../README.md#sessions-api)

The `status` method of the results API returns information about a sessions current status and progress.

## HTTP Request

`GET /api/sessions/<session_token>/status`

## Response Payload

```json
{
  "token": "String",
  "status": "Enum['pending', 'running', 'paused', 'completed', 'aborted']",
  "date_started": "String",
  "date_finished": "String",
  "expiration_date": "String"
}
```

- **token** contains the token of the session corresponding to this status.
- **status** specifies the current status of the session:
  - **pending**: The session was created, can receive updates, however cannot execute tests.
  - **running**: The session currently executes tests.
  - **paused**: The execution of tests in this session is currently paused.
  - **completed**: All tests files include in this session were executed and have a result.
  - **aborted**: The session was finished before all tests were executed.
- **date_started** contains the time the status changed from `PENDING` to `RUNNING` in ISO 8601.
- **date_finished** contains the time the status changed to either `COMPLETED` or `ABORTED` in ISO 8601.
- **expiration_date** contains the time at which the sessions will be deleted in ISO 8601.

## Example

**Request:**

`GET /api/sessions/d9caaae0-c362-11e9-943f-eedb305f22f6/status`

**Response:**

```json
{
  "token": "d9caaae0-c362-11e9-943f-eedb305f22f6",
  "status": "running",
  "date_started": "2019-09-04T14:21:19",
  "date_finished": null,
  "expiration_date": "2019-09-04T14:26:19"
}
```
