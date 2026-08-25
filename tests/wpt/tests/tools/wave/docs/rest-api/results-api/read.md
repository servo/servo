# `read` - [Results API](../README.md#results-api)

The `read` method of the results API returns all available results of a session, grouped by API. It is possible to filter the results to return by test directory or file.

## HTTP Request

`GET /api/results/<session_token>`

## Query Parameters

| Parameter | Description                    | Default | Example                     |
| --------- | ------------------------------ | ------- | --------------------------- |
| `path`    | Path of test directory or file | `/`     | `path=/apiOne/test/sub/dir` |

## Response Payload

```json
{
  "<api_name>": [
    {
      "test": "String",
      "status": "Enum['OK', 'ERROR', 'TIMEOUT', 'NOT_RUN']",
      "message": "String",
      "subtests": [
        {
          "name": "String",
          "status": "Enum['PASS', 'FAIL', 'TIMEOUT', 'NOT_RUN']",
          "message": "String"
        }
      ]
    }
  ]
}
```

Arrays of results grouped by their respective APIs. Structure of results is the same as described in the [`create`](./create.md) method of the results API.

## Example

**Request:**

`GET /api/results/974c84e0-c35d-11e9-8f8d-47bb5bb0037d?path=/apiOne/test/one.html`

**Response:**

```json
{
  "apiOne": [
    {
      "test": "/apiOne/test/one.html",
      "status": "OK",
      "message": null,
      "subtests": [
        {
          "name": "Value should be X",
          "status": "FAIL",
          "message": "Expected value to be X but got Y"
        }
      ]
    }
  ]
}
```
