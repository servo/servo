# `read next` - [Tests API](../README.md#tests-api)

The `read next` method of the tests API returns the next test of a test session, that is due to be executed. If the sessions status is not `RUNNING` it returns a static page containing information about the session and its current status.

## HTTP Request

`GET /api/tests/<session_token>/next`

## Response Payload

```json
{
  "next_test": "String"
}
```

## Example

**Request:**

`GET /api/tests/d6667670-c350-11e9-b504-4ac471cdd99d/next`

**Response:**

```json
{
  "next_test": "http://web-platform.test:8000/apiOne/test/one.html?&token=d6667670-c350-11e9-b504-4ac471cdd99d&timeout=60000"
}
```
