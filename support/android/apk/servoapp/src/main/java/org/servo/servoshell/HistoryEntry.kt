package org.servo.servoshell

import kotlinx.serialization.Serializable

@Serializable class HistoryEntry(val timestamp: Long, val url: String, val title: String?)
