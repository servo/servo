package org.servo.servoshell

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.core.view.isGone
import androidx.core.view.isVisible
import androidx.fragment.app.Fragment
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.MaterialToolbar
import org.servo.servoshell.HistoryAdapter.OnHistoryItemClickListener
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale

class HistoryFragment : Fragment(), OnHistoryItemClickListener {
    private lateinit var recyclerView: RecyclerView
    private lateinit var emptyState: View
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
        recyclerView = view.findViewById(R.id.history_recycler_view)
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

        recyclerView.setLayoutManager(LinearLayoutManager(requireContext()))

        loadHistory()
    }

    private fun loadHistory() {
        val historyEntries = historyManager.history

        if (historyEntries.isEmpty()) {
            recyclerView.isGone = true
            emptyState.isVisible = true
        } else {
            recyclerView.isVisible = true
            emptyState.isGone = true

            recyclerView.setAdapter(HistoryAdapter(groupByDay(historyEntries), this))
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

    override fun onHistoryItemClick(entry: HistoryEntry) {
        val resultIntent = Intent().apply {
            putExtra("url", entry.url)
        }
        requireActivity().setResult(Activity.RESULT_OK, resultIntent)
        requireActivity().finish()
    }
}
