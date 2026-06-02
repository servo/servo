# Import results - [Results API](../README.md#results-api)

If enabled, the WMAS Test Runner can import results exported by any arbitrary other instance.

## 1. Import session

Upload results and create a new, finished session

### HTTP Request

```
POST /api/results/import
```

#### HTTP Response

If successful, the server responds with the token of the imported session:

```json
{
  "token": "String"
}
```

However, if an error occured, the server responds the error message:

```json
{
  "error": "String"
}
```

## 2. Import API results

Upload a results json file and overwrite results of a specific API.

### HTTP Request

```
POST /api/results/<session_token>/<api_name>/json
```

### File structure

```json
{
  "results": [
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

### HTTP Response

If successful, the server responds with status code 200.
