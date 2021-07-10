# `read compact` - [Results API](../README.md#results-api)

The `read compact` method of the results API returns the number of passed, failed, timed out and not run tests per API of a session.

## HTTP Request

`GET /api/results/<session_token>/compact`

## Response Payload

```json
{
  "<api_name>": {
    "pass": "Integer",
    "fail": "Integer",
    "timeout": "Integer",
    "not_run": "Integer",
    "total": "Integer",
    "complete": "Integer"
  }
}
```

## Example

**Request:**

`GET /api/results/620bbf70-c35e-11e9-bf9c-742c02629054/compact`

**Response:**

```json
{
  "apiOne": {
    "pass": 311,
    "fail": 59,
    "timeout": 23,
    "not_run": 20,
    "total": 481,
    "complete": 413
  },
  "apiTwo": {
    "pass": 548,
    "fail": 129,
    "timeout": 53,
    "not_run": 36,
    "total": 766,
    "complete": 766
  },
  "apiThree": {
    "pass": 349,
    "fail": 45,
    "timeout": 14,
    "not_run": 9,
    "total": 523,
    "complete": 417
  }
}
```
