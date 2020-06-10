# `read available apis` - [Tests API](../README.md#tests-api)

The `read available apis` method return a list of all web APIs that the DUT 
can be tested for. It returns the human readable API name, as well as the 
directory name under which all corresponding tests reside.

## HTTP Request

`GET /api/tests/apis`

## Response Payload

```json
[
  {
    "path": "String",
    "name": "String"
  },
  ...
]
```

## Example

**Request:**

`GET /api/tests/apis`

**Response:**

```json
[
  {
    "path": "/2dcontext",
    "name": "2D Context"
  },
  {
    "path": "/media-source",
    "name": "Media Source"
  },
  ...
]
```
