# `send global event` - [Devices API](../README.md#devices-api)

The `send global event` method of the devices API enables sending an event to 
listeners of global device events.

## HTTP Request

`POST /api/devices/events`

## Request Payload

```json
{
    "type": "<String>",
    "data": "<Any>"
}
```

- **type** defines what type of event has been triggered.
- **data** contains the event specific payload.

## Event Types

See [global events](./event-types.md#global)

## Example

**Request:**

`POST /api/devices/1d9f5d30-830f-11ea-8dcb-0021ccd76152/events`

```json
{
    "type": "device_added",
    "data": {
        "token": "1d9f5d30-830f-11ea-8dcb-0021ccd76152",
        "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.113 Safari/537.36",
        "last_active": 1587391153295,
        "name": "Chrome 81.0.4044"
    }
}
```

**Response:**

`200 OK`
