# `read session` - [Tests API](../README.md#tests-api)

The `read session` method of the tests API fetches all tests contained in a test session grouped by their status.

## HTTP Request

`GET /api/tests/<session_token>`

## Response Payload

```json
{
  "token": "String",
  "pending_tests": {
    "<api_name>": "Array<String>"
  },
  "running_tests": {
    "<api_name>": "Array<String>"
  },
  "completed_tests": {
    "<api_name>": "Array<String>"
  }
}
```

- **pending_tests** are tests that have yet to be executed.
- **running_tests** are tests that are currently executed by the device under test. Although only one test at a time is executed, test that time out or fail to send a result may still wait for the time out to occur. In this case there are multiple tests in this list.
- **completed_tests** are tests that are finished and have a result.

## Example

**Request:**

`GET /api/tests/cd922410-c344-11e9-858f-9063f6dd878f`

**Response:**

```json
{
  "token": "cd922410-c344-11e9-858f-9063f6dd878f",
  "pending_tests": {
    "apiTwo": ["/apiTwo/test/three.html"],
    "apiThree": [
      "/apiThree/test/one.html",
      "/apiThree/test/two.html",
      "/apiThree/test/three.html"
    ]
  },
  "running_tests": {
    "apiTwo": ["/apiTwo/test/two.html"]
  },
  "completed_tests": {
    "apiOne": [
      "/apiOne/test/one.html",
      "/apiOne/test/two.html",
      "/apiOne/test/three.html"
    ],
    "apiTwo": ["/apiTwo/test/one.html"]
  }
}
```
