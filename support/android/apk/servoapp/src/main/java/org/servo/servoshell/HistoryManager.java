package org.servo.servoshell;

import android.content.Context;
import android.util.Log;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;


/**
 * Keeps a history of visited websites in an app-scoped JSON
 * file.
 */
public class HistoryManager {
    private static final String TAG = "HistoryManager";
    private static final String HISTORY_FILE = "history.json";

    private final File historyFile;

    public HistoryManager(Context context) {
        this.historyFile = new File(context.getFilesDir(), HISTORY_FILE);
    }

    /**
     * Add a new history entry.
     * Only adds if the URL is different from the most recent entry.
     */
    public synchronized void addEntry(String url, String title) {
        List<HistoryEntry> history = loadHistory();

        // "about:blank" sometimes pops up while loading pages, so filter that out
        if (url.equals("about:blank")) {
            return;
        }
        
        // Check if the URL is the same as the most recent entry. 
        // This avoids multiple entries, since Servo currently fires
        // multiple onLoadEnded events per page.
        if (!history.isEmpty()) {
            HistoryEntry mostRecent = history.get(0);
            if (mostRecent.getUrl().equals(url)) {
                Log.d(TAG, "Skipping duplicate URL: " + url);
                return;
            }
        }
        
        long timestamp = System.currentTimeMillis();
        HistoryEntry entry = new HistoryEntry(timestamp, url, title);
        
        // We sort the history most recent first, so new stuff at 
        // the beginning
        history.add(0, entry);
        
        saveHistory(history);
    }

    /**
     * Get all history entries
     */
    public synchronized List<HistoryEntry> getHistory() {
        return loadHistory();
    }

    /**
     * Clear all history
     */
    public synchronized void clearHistory() {
        if (historyFile.exists()) {
            historyFile.delete();
        }
        Log.i(TAG, "History cleared");
    }

    /**
     * Load history from JSON file
     */
    private List<HistoryEntry> loadHistory() {
        List<HistoryEntry> history = new ArrayList<>();
        
        if (!historyFile.exists()) {
            return history;
        }

        try {
            // Read the JSON line by line and reassemble it into
            // a big string, then parse it as JSON.
            StringBuilder json = new StringBuilder();
            BufferedReader reader = new BufferedReader(new FileReader(historyFile));
            String line;
            while ((line = reader.readLine()) != null) {
                json.append(line);
            }
            reader.close();

            JSONArray jsonArray = new JSONArray(json.toString());
            // Populate the history
            for (int i = 0; i < jsonArray.length(); i++) {
                JSONObject jsonObject = jsonArray.getJSONObject(i);
                history.add(HistoryEntry.fromJSON(jsonObject));
            }
        } catch (IOException | JSONException e) {
            Log.e(TAG, "Error loading history", e);
        }

        return history;
    }

    /**
     * Save history to JSON file
     */
    private void saveHistory(List<HistoryEntry> history) {
        try {
            JSONArray jsonArray = new JSONArray();
            for (HistoryEntry entry : history) {
                jsonArray.put(entry.toJSON());
            }

            FileWriter writer = new FileWriter(historyFile);
            // Pretty print with indent
            writer.write(jsonArray.toString(2));
            writer.close();

            Log.d(TAG, "History saved to JSON");
        } catch (IOException | JSONException e) {
            Log.e(TAG, "Error saving history", e);
        }
    }
}
