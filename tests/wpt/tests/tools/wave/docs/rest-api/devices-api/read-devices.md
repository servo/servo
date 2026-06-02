# `read devices` - [Devices API](../README.md#devices-api)

The `read devices` method of the devices API fetches a list of all registered 
devices.

## HTTP Request

`GET /api/devices`

## Response Payload

```json
[
  {
      "token": "<String>",
      "user_agent": "<String>",
      "last_active": "<String>",
      "name": "<String>"
  },
  ...
]
```

- **token** is the unique identifier of the device.
- **user_agent** is the user agent of the request the device was registered with.
- **last_active** defines the point in time the device was last active. Expressed as ISO 8601 date and time format.
- **name** the name the device was assign based on its user agent.

## Example

**Request:**

`GET /api/devices`

**Response:**

```json
[
  {
      "token": "1d9f5d30-830f-11ea-8dcb-0021ccd76152",
      "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.113 Safari/537.36",
      "last_active": 1587391153295,
      "name": "Chrome 81.0.4044"
  },
  ...
]
```
