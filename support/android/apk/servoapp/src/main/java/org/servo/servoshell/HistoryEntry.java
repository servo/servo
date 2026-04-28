package org.servo.servoshell;

import org.json.JSONException;
import org.json.JSONObject;

public class HistoryEntry {
    private long timestamp;
    private String url;
    private String title;

    public HistoryEntry(long timestamp, String url, String title) {
        this.timestamp = timestamp;
        this.url = url;
        this.title = title;
    }

    public long getTimestamp() {
        return timestamp;
    }

    public String getUrl() {
        return url;
    }

    public String getTitle() {
        return title;
    }

    // Convert to JSON
    public JSONObject toJSON() throws JSONException {
        JSONObject json = new JSONObject();
        json.put("timestamp", timestamp);
        json.put("url", url);
        json.put("title", title != null ? title : "");
        return json;
    }

    // Create from JSON
    public static HistoryEntry fromJSON(JSONObject json) throws JSONException {
        long timestamp = json.getLong("timestamp");
        String url = json.getString("url");
        String title = json.optString("title", "");
        return new HistoryEntry(timestamp, url, title);
    }
}
