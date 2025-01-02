# `labels` - [Sessions API](../README.md#sessions-api)

The `labels` methods of the sessions API allow for better organization of sessions.

## Read labels

Reads all labels of a session.

### HTTP Request

`GET /api/sessions/<token>/labels`

### Response Payload

```json
"Array<String>"
```

#### Example

**Request:**

`GET /api/sessions/afd4ecb0-c339-11e9-b66c-eca76c2bea9c/labels`

**Response:**

```json
["label1", "label2", "label3"]
```

## Update labels

Update all labels of a session.

### HTTP Request

`PUT /api/sessions/<token>/labels`

### Request Payload

```json
"Array<String>"
```

The array of labels provided in the request payload will replace all existing labels of the session.

#### Example

**Request:**

`GET /api/sessions/afd4ecb0-c339-11e9-b66c-eca76c2bea9c/labels`

**Response:**

```json
["label1", "label2", "label3"]
```

**Request:**

`PUT /api/sessions/afd4ecb0-c339-11e9-b66c-eca76c2bea9c/labels`

```json
["label4", "label5"]
```

**Request:**

`GET /api/sessions/afd4ecb0-c339-11e9-b66c-eca76c2bea9c/labels`

**Response:**

```json
["label4", "label5"]
```
