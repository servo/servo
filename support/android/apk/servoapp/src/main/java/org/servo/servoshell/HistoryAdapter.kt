package org.servo.servoshell

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class HistoryAdapter(
    private val items: List<HistoryItem>,
    private val clickListener: OnHistoryItemClickListener,
) : RecyclerView.Adapter<RecyclerView.ViewHolder>() {
    private val timeFormat = SimpleDateFormat("HH:mm", Locale.getDefault())

    interface OnHistoryItemClickListener {
        fun onHistoryItemClick(entry: HistoryEntry)
    }

    override fun getItemViewType(position: Int): Int = items[position].type

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RecyclerView.ViewHolder {
        if (viewType == HistoryItem.TYPE_HEADER) {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.history_header, parent, false)
            return HeaderViewHolder(view)
        } else {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.history_item, parent, false)
            return EntryViewHolder(view)
        }
    }

    override fun onBindViewHolder(holder: RecyclerView.ViewHolder, position: Int) {
        val item = items[position]

        if (item.type == HistoryItem.TYPE_HEADER) {
            val headerHolder = holder as HeaderViewHolder
            val headerItem = item as HistoryHeaderItem

            headerHolder.headerText.text = headerItem.headerText
        } else {
            val entryHolder = holder as EntryViewHolder
            val entryItem = item as HistoryEntryItem
            val entry = entryItem.entry

            entryHolder.titleView.text = entry.title?.takeUnless { it.isEmpty() } ?: entry.url

            entryHolder.urlView.text = entry.url

            entryHolder.timeView.text = timeFormat.format(Date(entry.timestamp))

            entryHolder.itemView.setOnClickListener {
                clickListener.onHistoryItemClick(entry)
            }
        }
    }

    override fun getItemCount(): Int = items.size

    private class HeaderViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        val headerText = itemView.findViewById<TextView>(R.id.history_header_text)!!
    }

    private class EntryViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        val titleView = itemView.findViewById<TextView>(R.id.history_item_title)!!
        val urlView = itemView.findViewById<TextView>(R.id.history_item_url)!!
        val timeView = itemView.findViewById<TextView>(R.id.history_item_time)!!
    }
}
