# `create` - [Results API](../README.md#results-api)

The `create` method of the results API creates a test result for a given test of a test session.

## HTTP Request

`POST /api/results/<session_token>`

## Request Payload

```json
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
```

- **test** specifies the test to create the result for.
- **status** specifies the overall status of the test. It does not represent a result, but rather if the contained tests were executed as intended or if something went wrong running the test.
  - **OK**: All tests were executed without problems.
  - **ERROR**: There was an error running one or multiple tests.
  - **TIMEOUT**: It took too long for the tests to execute.
  - **NOT_RUN**: This test was skipped.
- **message** contains the reason for the overall status. If the status is `OK` the message should be `null`.
- **subtests** contains the actual results of the tests executed in this file.
  - **name**: The name of the test.
  - **status**: The status of the result:
    - **PASS**: The test was executed successfully.
    - **FAIL**: The test did not meet at least one assertion.
    - **TIMEOUT**: It took too long for this test to execute.
    - **NOT_RUN**: This test was skipped.
  - **message** contains the reason for the tests failure.

## Example

**Request:**

`POST /api/results/d89bcc00-c35b-11e9-8bb7-9e3d7595d40c`

```json
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
```

**Response:**

`200 OK`