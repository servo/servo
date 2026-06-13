package org.servo.servoshell

import org.json.JSONException
import org.json.JSONObject

class HistoryEntry(val timestamp: Long, val url: String, val title: String?) {
    @Throws(JSONException::class)
    fun toJSON() = JSONObject().apply {
        put("timestamp", timestamp)
        put("url", url)
        put("title", title.orEmpty())
    }

    companion object {
        @JvmStatic
        @Throws(JSONException::class)
        fun fromJSON(json: JSONObject) = HistoryEntry(
            timestamp = json.getLong("timestamp"),
            url = json.getString("url"),
            title = json.optString("title", ""),
        )
    }
}
