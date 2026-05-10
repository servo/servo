package org.servo.servoshell;

// Genric history list item. This can be
// either a header (which separate the proper entries by day),
// or an actual browser history entry with 
// the time, URL and title.
public interface HistoryItem {
    int TYPE_HEADER = 0; // HistoryHeaderItem.java + history_header.xml
    int TYPE_ENTRY = 1;  // HistoryEntryItem.java + history_item.xml
    
    int getType();
}
