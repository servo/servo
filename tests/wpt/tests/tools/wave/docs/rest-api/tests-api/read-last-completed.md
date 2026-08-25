# `read last completed` - [Tests API](../README.md#tests-api)

The `read last completed` method of the tests API returns a list of test files, which most recently finished and have a result. The files are grouped by the status their respective result had.

## HTTP Request

`GET /api/tests/<session_token>/last_completed`

## Query Parameters

| Parameter | Desciption                                                                                                                | Default | Example               |
| --------- | ------------------------------------------------------------------------------------------------------------------------- | ------- | --------------------- |
| `count`   | Number of files per status to return                                                                                      | 5       | `count=5`             |
| `status`  | The status the files results must have. Comma separated list. Possible values: `all`, `pass`, `fail` and `timeout` | `all`   | `status=timeout,pass` |

## Response Payload

```json
{
  "pass": "Array<String>",
  "fail": "Array<String>",
  "timeout": "Array<String>"
}
```

## Example

**Request:**

`GET /api/tests/7dafeec0-c351-11e9-84c5-3d1ede2e7d2e/last_completed?count=3&status=fail,timeout`

**Response:**

```json
{
  "fail": [
    "/apiTwo/test/four.html",
    "/apiOne/test/twentyfour.html",
    "/apiOne/test/nineteen.html"
  ],
  "timeout": [
    "/apiFive/test/eight.html",
    "/apiThree/test/five.html",
    "/apiThree/test/two.html"
  ]
}
```
