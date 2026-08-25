# Device Event Types - [Devices API](../README.md#devices-api)

Device events are events that are triggered by actions related to devices. 
This document specifies what possible events can occur and what they mean.

## Device specific <a name="device-specific"></a>

Device specific events are always related to a specific device, referenced by 
its token.

### Start session

**Type identifier**: `start_session`  
**Payload**: 
```json
{
  "session_token": "<String>"
}
```
**Description**: Triggered by a companion device, this event starts a 
pre-configured session on the registered device.

## Global <a name="global"></a>

Global device events have no special relation to any device.

### Device added

**Type identifier**: `device_added`  
**Payload**: Same as response of [`read device`](./read-device.md) method.  
**Description**: This event is triggered once a new device registers.    

### Device removed

**Type identifier**: `device_removed`  
**Payload**: Same as response of [`read device`](./read-device.md) method.  
**Description**: This event is triggered once a device unregisters.    
