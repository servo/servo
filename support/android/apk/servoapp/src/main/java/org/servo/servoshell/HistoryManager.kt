package org.servo.servoshell

import android.content.Context
import android.util.Log
import org.json.JSONArray
import org.json.JSONException
import org.servo.servoshell.HistoryEntry.Companion.fromJSON
import java.io.BufferedReader
import java.io.File
import java.io.FileReader
import java.io.FileWriter
import java.io.IOException

/**
 * Keeps a history of visited websites in an app-scoped JSON
 * file.
 */
class HistoryManager(context: Context) {
    private val historyFile = File(context.filesDir, HISTORY_FILE)

    /**
     * Add a new history entry.
     * Only adds if the URL is different from the most recent entry.
     */
    fun addEntry(url: String, title: String?) {
        val history = loadHistory()

        // "about:blank" sometimes pops up while loading pages, so filter that out
        if (url == "about:blank") {
            return
        }

        // Check if the URL is the same as the most recent entry.
        // This avoids multiple entries, since Servo currently fires
        // multiple onLoadEnded events per page.
        if (history.isNotEmpty()) {
            val mostRecent = history.first()
            if (mostRecent.url == url) {
                Log.d(TAG, "Skipping duplicate URL: $url")
                return
            }
        }

        val timestamp = System.currentTimeMillis()
        val entry = HistoryEntry(timestamp, url, title)

        // We sort the history most recent first, so new stuff at 
        // the beginning
        history.add(0, entry)

        saveHistory(history)
    }

    val history: MutableList<HistoryEntry>
        get() = loadHistory()

    /**
     * Clear all history
     */
    fun clearHistory() {
        if (historyFile.exists()) {
            historyFile.delete()
        }
        Log.i(TAG, "History cleared")
    }

    /**
     * Load history from JSON file
     */
    private fun loadHistory(): MutableList<HistoryEntry> {
        val history = mutableListOf<HistoryEntry>()

        if (!historyFile.exists()) {
            return history
        }

        try {
            val jsonArray = JSONArray(historyFile.readText())
            for (i in 0..<jsonArray.length()) {
                val jsonObject = jsonArray.getJSONObject(i)
                history.add(fromJSON(jsonObject))
            }
        } catch (e: IOException) {
            Log.e(TAG, "Error loading history", e)
        } catch (e: JSONException) {
            Log.e(TAG, "Error loading history", e)
        }

        return history
    }

    /**
     * Save history to JSON file
     */
    private fun saveHistory(history: MutableList<HistoryEntry>) {
        try {
            val jsonArray = JSONArray()
            for (entry in history) {
                jsonArray.put(entry.toJSON())
            }

            FileWriter(historyFile).use { writer ->
                // Pretty print with indent
                writer.write(jsonArray.toString(2))
            }

            Log.d(TAG, "History saved to JSON")
        } catch (e: IOException) {
            Log.e(TAG, "Error saving history", e)
        } catch (e: JSONException) {
            Log.e(TAG, "Error saving history", e)
        }
    }

    companion object {
        private const val TAG = "HistoryManager"
        private const val HISTORY_FILE = "history.json"
    }
}
