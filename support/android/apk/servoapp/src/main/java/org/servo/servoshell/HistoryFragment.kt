package org.servo.servoshell

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.view.isGone
import androidx.core.view.isVisible
import androidx.fragment.app.Fragment
import com.google.android.material.appbar.MaterialToolbar
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale

class HistoryFragment : Fragment() {
    private val timeFormat = SimpleDateFormat("HH:mm", Locale.getDefault())

    private lateinit var listView: ComposeView
    private lateinit var emptyState: ComposeView
    private lateinit var historyManager: HistoryManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        historyManager = HistoryManager(requireContext())
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View = inflater.inflate(R.layout.fragment_history, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        listView = view.findViewById(R.id.history_list)
        emptyState = view.findViewById(R.id.empty_state)

        val toolbar = view.findViewById<MaterialToolbar>(R.id.toolbar)
        toolbar.setOnMenuItemClickListener { item ->
            if (item.itemId == R.id.clear_history) {
                clearHistory()
                true
            } else {
                false
            }
        }

        loadHistory()
    }

    private fun loadHistory() {
        val historyEntries = historyManager.history

        if (historyEntries.isEmpty()) {
            listView.isGone = true
            emptyState.isVisible = true

            emptyState.setContent {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        stringResource(R.string.history_placeholder_title),
                        style = MaterialTheme.typography.headlineSmall,
                    )
                    Text(
                        stringResource(R.string.history_placeholder_message),
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        } else {
            listView.isVisible = true
            emptyState.isGone = true

            listView.setContent {
                LazyColumn {
                    items(groupByDay(historyEntries)) { item ->
                        when (item) {
                            is HistoryHeaderItem -> {
                                ListItem(
                                    headlineContent = {
                                        Text(
                                            item.headerText,
                                            style = MaterialTheme.typography.titleSmall,
                                        )
                                    },
                                )
                            }
                            is HistoryEntryItem -> {
                                ListItem(
                                    modifier = Modifier
                                        .clickable {
                                            val resultIntent = Intent().apply {
                                                putExtra("url", item.entry.url)
                                            }
                                            requireActivity().setResult(Activity.RESULT_OK, resultIntent)
                                            requireActivity().finish()
                                        },
                                    headlineContent = {
                                        Text(
                                            item.entry.title?.takeUnless { it.isEmpty() } ?: item.entry.url,
                                            overflow = TextOverflow.Ellipsis,
                                            maxLines = 1,
                                        )
                                    },
                                    supportingContent = {
                                        Text(
                                            item.entry.url,
                                            overflow = TextOverflow.Ellipsis,
                                            maxLines = 1,
                                        )
                                    },
                                    leadingContent = {
                                        Text(timeFormat.format(Date(item.entry.timestamp)))
                                    },
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    private fun groupByDay(entries: List<HistoryEntry>): List<HistoryItem> {
        val items = mutableListOf<HistoryItem>()

        if (entries.isEmpty()) {
            return items
        }

        val dayFormat = SimpleDateFormat("EEEE, MMMM d", Locale.getDefault())

        val currentCal = Calendar.getInstance()
        val entryCal = Calendar.getInstance()
        val todayCal = Calendar.getInstance()

        todayCal.set(Calendar.HOUR_OF_DAY, 0)
        todayCal.set(Calendar.MINUTE, 0)
        todayCal.set(Calendar.SECOND, 0)
        todayCal.set(Calendar.MILLISECOND, 0)

        var lastDay: String? = null

        for (entry in entries) {
            entryCal.setTimeInMillis(entry.timestamp)

            currentCal.setTimeInMillis(entry.timestamp)
            currentCal.set(Calendar.HOUR_OF_DAY, 0)
            currentCal.set(Calendar.MINUTE, 0)
            currentCal.set(Calendar.SECOND, 0)
            currentCal.set(Calendar.MILLISECOND, 0)

            var dayHeader = dayFormat.format(Date(entry.timestamp))
            if (currentCal.getTimeInMillis() == todayCal.getTimeInMillis()) {
                dayHeader = "Today – $dayHeader"
            }

            if (dayHeader != lastDay) {
                items.add(HistoryHeaderItem(dayHeader))
                lastDay = dayHeader
            }

            items.add(HistoryEntryItem(entry))
        }

        return items
    }

    private fun clearHistory() {
        historyManager.clearHistory()
        loadHistory()
    }
}
