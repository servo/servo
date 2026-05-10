package org.servo.servoshell;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import com.google.android.material.appbar.MaterialToolbar;

import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Calendar;
import java.util.Date;
import java.util.List;
import java.util.Locale;

public class HistoryFragment extends Fragment implements HistoryAdapter.OnHistoryItemClickListener {
    
    private RecyclerView recyclerView;
    private View emptyState;
    private HistoryManager historyManager;
    
    @Override
    public void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        historyManager = new HistoryManager(requireContext());
    }
    
    @Nullable
    @Override
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container,
                             @Nullable Bundle savedInstanceState) {
        // Inflate the Material3 layout
        return inflater.inflate(R.layout.fragment_history, container, false);
    }
    
    @Override
    public void onViewCreated(@NonNull View view, @Nullable Bundle savedInstanceState) {
        super.onViewCreated(view, savedInstanceState);
        
        recyclerView = view.findViewById(R.id.history_recycler_view);
        emptyState = view.findViewById(R.id.empty_state);
        
        // Set up toolbar menu
        MaterialToolbar toolbar = view.findViewById(R.id.toolbar);
        toolbar.setOnMenuItemClickListener(item -> {
            if (item.getItemId() == R.id.clear_history) {
                clearHistory();
                return true;
            }
            return false;
        });
        
        // Set up RecyclerView
        recyclerView.setLayoutManager(new LinearLayoutManager(requireContext()));
        
        // Load and display history
        loadHistory();
    }
    
    private void loadHistory() {
        List<HistoryEntry> historyEntries = historyManager.getHistory();
        
        if (historyEntries.isEmpty()) {
            // Show empty state
            recyclerView.setVisibility(View.GONE);
            emptyState.setVisibility(View.VISIBLE);
        } else {
            // Show history list
            recyclerView.setVisibility(View.VISIBLE);
            emptyState.setVisibility(View.GONE);
            
            // Group entries by day with headers
            List<HistoryItem> items = groupByDay(historyEntries);
            
            HistoryAdapter adapter = new HistoryAdapter(items, this);
            recyclerView.setAdapter(adapter);
        }
    }
    
    private List<HistoryItem> groupByDay(List<HistoryEntry> entries) {
        List<HistoryItem> items = new ArrayList<>();
        
        if (entries.isEmpty()) {
            return items;
        }
        
        SimpleDateFormat dayFormat = new SimpleDateFormat("EEEE, MMMM d", Locale.getDefault());

        Calendar currentCal = Calendar.getInstance();
        Calendar entryCal = Calendar.getInstance();
        Calendar todayCal = Calendar.getInstance();
        
        // Reset time portion for date comparison
        todayCal.set(Calendar.HOUR_OF_DAY, 0);
        todayCal.set(Calendar.MINUTE, 0);
        todayCal.set(Calendar.SECOND, 0);
        todayCal.set(Calendar.MILLISECOND, 0);
        
        String lastDay = null;
        
        for (HistoryEntry entry : entries) {
            entryCal.setTimeInMillis(entry.getTimestamp());
            
            // Reset time portion for comparison
            currentCal.setTimeInMillis(entry.getTimestamp());
            currentCal.set(Calendar.HOUR_OF_DAY, 0);
            currentCal.set(Calendar.MINUTE, 0);
            currentCal.set(Calendar.SECOND, 0);
            currentCal.set(Calendar.MILLISECOND, 0);
            
            // Format day header
            String dayHeader = dayFormat.format(new Date(entry.getTimestamp()));
            if (currentCal.getTimeInMillis() == todayCal.getTimeInMillis()) {
                // Prefix "Today" if it’s… today. 
                dayHeader = "Today – " + dayHeader;
            }
            
            // Add header if this is a new day
            if (!dayHeader.equals(lastDay)) {
                items.add(new HistoryHeaderItem(dayHeader));
                lastDay = dayHeader;
            }
            
            // Add entry
            items.add(new HistoryEntryItem(entry));
        }
        
        return items;
    }
    
    private void clearHistory() {
        historyManager.clearHistory();
        // Refresh history page
        loadHistory();
    }
    
    @Override
    public void onHistoryItemClick(HistoryEntry entry) {
        // Send the selected URL back to the MainActivity
        Intent resultIntent = new Intent();
        resultIntent.putExtra("url", entry.getUrl());
        requireActivity().setResult(Activity.RESULT_OK, resultIntent);
        requireActivity().finish();
    }
}
