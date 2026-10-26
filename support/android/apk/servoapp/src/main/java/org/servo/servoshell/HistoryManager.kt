package org.servo.servoshell

import android.content.Context
import android.util.Log
import java.io.File
import java.io.IOException
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json

/** Keeps a history of visited websites in an app-scoped JSON file. */
class HistoryManager(context: Context) {
    private val historyFile = File(context.filesDir, HISTORY_FILE)

    /** Add a new history entry. Only adds if the URL is different from the most recent entry. */
    fun addEntry(url: String, title: String?) {
        val history = loadHistory().toMutableList()

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

    val history: List<HistoryEntry>
        get() = loadHistory()

    /** Clear all history */
    fun clearHistory() {
        if (historyFile.exists()) {
            historyFile.delete()
        }
        Log.i(TAG, "History cleared")
    }

    /** Load history from JSON file */
    private fun loadHistory(): List<HistoryEntry> {
        if (!historyFile.exists()) {
            return emptyList()
        }

        try {
            return Json.decodeFromString<List<HistoryEntry>>(historyFile.readText())
        } catch (e: IOException) {
            Log.e(TAG, "Error loading history", e)
        } catch (e: SerializationException) {
            Log.e(TAG, "Error loading history", e)
        }

        return emptyList()
    }

    /** Save history to JSON file */
    private fun saveHistory(history: List<HistoryEntry>) {
        try {
            historyFile.writeText(Json.encodeToString(history))

            Log.d(TAG, "History saved to JSON")
        } catch (e: IOException) {
            Log.e(TAG, "Error saving history", e)
        } catch (e: SerializationException) {
            Log.e(TAG, "Error saving history", e)
        }
    }

    companion object {
        private const val TAG = "HistoryManager"
        private const val HISTORY_FILE = "history.json"
    }
}
