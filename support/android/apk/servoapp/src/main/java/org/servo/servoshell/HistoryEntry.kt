package org.servo.servoshell

import org.json.JSONException
import org.json.JSONObject

class HistoryEntry(val timestamp: Long, val url: String, val title: String?) {
    @Throws(JSONException::class)
    fun toJSON() =
        JSONObject().apply {
            put("timestamp", timestamp)
            put("url", url)
            putOpt("title", title)
        }

    companion object {
        @Throws(JSONException::class)
        fun fromJSON(json: JSONObject) =
            HistoryEntry(
                timestamp = json.getLong("timestamp"),
                url = json.getString("url"),
                title = if (json.has("title")) json.getString("title") else null,
            )
    }
}
