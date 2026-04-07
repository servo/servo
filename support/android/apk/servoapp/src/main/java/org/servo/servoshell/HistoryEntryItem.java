package org.servo.servoshell;

public class HistoryEntryItem implements HistoryItem {
    private final HistoryEntry entry;
    
    public HistoryEntryItem(HistoryEntry entry) {
        this.entry = entry;
    }
    
    public HistoryEntry getEntry() {
        return entry;
    }
    
    @Override
    public int getType() {
        return TYPE_ENTRY;
    }
}
