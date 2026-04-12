package org.servo.servoshell;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.List;
import java.util.Locale;

public class HistoryAdapter extends RecyclerView.Adapter<RecyclerView.ViewHolder> {

    private final List<HistoryItem> items;
    private final OnHistoryItemClickListener clickListener;
    private final SimpleDateFormat timeFormat;

    public interface OnHistoryItemClickListener {
        void onHistoryItemClick(HistoryEntry entry);
    }

    public HistoryAdapter(List<HistoryItem> items, OnHistoryItemClickListener clickListener) {
        this.items = items;
        this.clickListener = clickListener;
        this.timeFormat = new SimpleDateFormat("HH:mm", Locale.getDefault());
    }

    @Override
    public int getItemViewType(int position) {
        return items.get(position).getType();
    }

    @NonNull
    @Override
    public RecyclerView.ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        if (viewType == HistoryItem.TYPE_HEADER) {
            View view = LayoutInflater.from(parent.getContext()).inflate(R.layout.history_header, parent, false);
            return new HeaderViewHolder(view);
        } else {
            View view = LayoutInflater.from(parent.getContext()).inflate(R.layout.history_item, parent, false);
            return new EntryViewHolder(view);
        }
    }

    @Override
    public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position) {
        HistoryItem item = items.get(position);
        
        if (item.getType() == HistoryItem.TYPE_HEADER) {
            HeaderViewHolder headerHolder = (HeaderViewHolder) holder;
            HistoryHeaderItem headerItem = (HistoryHeaderItem) item;
            headerHolder.headerText.setText(headerItem.getHeaderText());
        } else {
            EntryViewHolder entryHolder = (EntryViewHolder) holder;
            HistoryEntryItem entryItem = (HistoryEntryItem) item;
            HistoryEntry entry = entryItem.getEntry();
            
            // Set title, or use URL if title is empty
            String title = entry.getTitle();
            if (title == null || title.isEmpty()) {
                title = entry.getUrl();
            }
            entryHolder.titleView.setText(title);
            
            // Set URL
            entryHolder.urlView.setText(entry.getUrl());
            
            // Set time (just HH:mm format)
            String time = timeFormat.format(new Date(entry.getTimestamp()));
            entryHolder.timeView.setText(time);
            
            // Set click listener
            entryHolder.itemView.setOnClickListener(v -> {
                if (clickListener != null) {
                    clickListener.onHistoryItemClick(entry);
                }
            });
        }
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    static class HeaderViewHolder extends RecyclerView.ViewHolder {
        TextView headerText;

        HeaderViewHolder(@NonNull View itemView) {
            super(itemView);
            headerText = itemView.findViewById(R.id.history_header_text);
        }
    }

    static class EntryViewHolder extends RecyclerView.ViewHolder {
        TextView titleView;
        TextView urlView;
        TextView timeView;

        EntryViewHolder(@NonNull View itemView) {
            super(itemView);
            titleView = itemView.findViewById(R.id.history_item_title);
            urlView = itemView.findViewById(R.id.history_item_url);
            timeView = itemView.findViewById(R.id.history_item_time);
        }
    }
}
