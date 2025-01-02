# `config` - [Results API](../README.md#results-api)

The `config` method is used to determine what features of the results API are 
enabled. Features that can be enabled or disabled are the 
[`import`](./import.md) method and the generation of reports and therefore 
[`download and view`](./download.md) methods.

## HTTP Request

`GET /api/results/config`

## Response

```json
{
  "import_enabled": "Boolean",
  "reports_enabled": "Boolean"
}
```

## Example

**Request:**

`GET /api/results/config`

**Response:**

```json
{
  "import_enabled": false,
  "reports_enabled": true
}
```
