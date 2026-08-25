# `delete` - [Sessions API](../README.md#sessions-api)

The `delete` method of the sessions API is used to delete a session and single results associated with it. However artifacts like generated reports or JSON files containing results of a whole API remain, therefore urls to those resources are still working.

## HTTP Request

`DELETE /api/sessions/<session_token>`

## Example

**Request:**

`DELETE /api/sessions/1592b880-c339-11e9-b414-61af09c491b1`

**Response:**

`200 OK`

**Request:**

`GET /api/sessions/1592b880-c339-11e9-b414-61af09c491b1`

**Response:**

`404 NOT FOUND`
