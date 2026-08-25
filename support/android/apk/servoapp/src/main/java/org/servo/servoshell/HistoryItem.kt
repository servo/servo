package org.servo.servoshell

sealed interface HistoryItem

class HistoryHeaderItem(val headerText: String) : HistoryItem

class HistoryEntryItem(val entry: HistoryEntry) : HistoryItem
