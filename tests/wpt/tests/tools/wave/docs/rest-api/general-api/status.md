# `status` - [General API](../README.md#general-api)

The `status` method is used to ensure the server is reachable and to determine 
what features of different server APIs are enabled. 

## HTTP Request

```
GET /api/status
```

### Response

```json
{
  "version_string": "String",
  "import_results_enabled": "Boolean",
  "reports_enabled": "Boolean",
  "read_sessions_enabled": "Boolean"
}
```

- **version_string**: The version of the server.
- **import_results_enabled**: If true the [`import result`](../results-api/import.md) endpoint is available
- **reports_enabled**: If true the server will generate reports for completed APIs in a given test session.
- **read_sessions_enabled**: If true it is possible to list all sessions using the [`read sessions`](../sessions-api/read_sessions.md) endpoint of the sessions API

## Example

```
GET /api/status
```

```json
{
  "version_string": "v2.0.0",
  "import_results_enabled": false,
  "reports_enabled": true,
  "read_sessions_enabled": false
}
```
