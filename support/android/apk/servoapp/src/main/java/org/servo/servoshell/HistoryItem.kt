package org.servo.servoshell

sealed interface HistoryItem {
    val type: Int

    companion object {
        const val TYPE_HEADER = 0
        const val TYPE_ENTRY = 1
    }
}

class HistoryHeaderItem(val headerText: String) : HistoryItem {
    override val type get() = HistoryItem.TYPE_HEADER
}

class HistoryEntryItem(val entry: HistoryEntry) : HistoryItem {
    override val type get() = HistoryItem.TYPE_ENTRY
}
