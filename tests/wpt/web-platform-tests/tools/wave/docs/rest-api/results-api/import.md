# Import results - [Results API](../README.md#results-api)

If enabled, the WMAS Test Suite can import results exported by any arbitrary other instance.

## 1. `import`

Import a session's results from a ZIP file.

### HTTP Request

`POST /api/results/import`

### HTTP Response

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

## 2. `import enabled`

To check whether or not the import features is enabled, the `import enabled` method returns the state as JSON.

### HTTP Request

`GET /api/results/import`

### Response

```json
{
  "enabled": "Boolean"
}
```
