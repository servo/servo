# `read sessions` - [Sessions API](../README.md#sessions-api)

The `read sessions` method of the sessions API fetches a list of all available
session's token with the option expand the returned data by all retrieved
tokens corresponding session configurations or statuses.

## HTTP Request

```
GET /api/sessions
```

### Query Parameters

| Parameter | Description                                                                                          | Default |
| --------- | ---------------------------------------------------------------------------------------------------- | ------- |
| `index`   | At what index of all session to start the returned list.                                             | `0`     |
| `count`   | How many entries to return starting from `index`.                                                    | `10`    |
| `expand`  | Comma separated list of relations from `_links`. Includes additional data in the `_embedded` object. | none    |

### Response Payload

```
200 OK
Content-Type: application/json+hal
```

```json
{
  "_links": {
    "<relation>": {
      "href": "String"
    }
    ...
  },
  "_embedded": {
    "<relation>": "Array<Any>"
    ...
  },
  "items": "Array<String>"
}
```

- **\_links** contains URLs to related data. For more, see [HAL Specfication](https://tools.ietf.org/html/draft-kelly-json-hal).
- **\_embedded** additional content specified by `expand` query paramater. For more, see [HAL Specfication](https://tools.ietf.org/html/draft-kelly-json-hal).
- **items** contains the returned list of session tokens.

## Example

```
GET /api/sessions?index=0&count=3&expand=status
```

```
200 OK
Content-Type: application/json+hal
```

```json
{
  "_links": {
    "first": {
      "href": "/_wave/api/sessions?index=0&count=3"
    },
    "last": {
      "href": "/_wave/api/sessions?index=39&count=3"
    },
    "self": {
      "href": "/_wave/api/sessions?index=0&count=3"
    },
    "next": {
      "href": "/_wave/api/sessions?index=3&count=3"
    },
    "configuration": {
      "href": "/_wave/api/sessions/{token}",
      "templated": true
    },
    "status": {
      "href": "/_wave/api/sessions/{token}/status",
      "templated": true
    }
  },
  "items": [
    "13f80c84-9046-11ea-9c80-0021ccd76152",
    "34db08e4-903b-11ea-89ce-0021ccd76152",
    "a355f846-9465-11ea-ae9e-0021ccd76152"
  ],
  "_embedded": {
    "status": [
      {
        "status": "completed",
        "expiration_date": null,
        "labels": [],
        "date_finished": 1588844145897.157,
        "token": "13f80c84-9046-11ea-9c80-0021ccd76152",
        "date_created": null,
        "date_started": 1588844127000,
        "type": "wmas"
      },
      {
        "status": "completed",
        "expiration_date": null,
        "labels": [],
        "date_finished": 1588839822768.9568,
        "token": "34db08e4-903b-11ea-89ce-0021ccd76152",
        "date_created": null,
        "date_started": 1588839522000,
        "type": "wmas"
      },
      {
        "status": "completed",
        "expiration_date": null,
        "labels": [],
        "date_finished": 1589297485065.1802,
        "token": "a355f846-9465-11ea-ae9e-0021ccd76152",
        "date_created": null,
        "date_started": 1589297484000,
        "type": "wmas"
      }
    ]
  }
}
```
