# `create` - [Devices API](../README.md#devices-api)

The `create` method of the devices API registers a new devices to remotely 
start test sessions on. The device will automatically be unregistered if its 
not registering for an [event](./register.md) for more than a minute.

## HTTP Request

`POST /api/devices`

## Response Payload

```json
{
    "token": "<String>"
}
```

- **token** specifies the handle to reference the registered device by.

## Example

**Request:**

`POST /api/devices`

**Response:**

```json
{
    "token": "e5f0b92e-8309-11ea-a1b1-0021ccd76152"
}
```
