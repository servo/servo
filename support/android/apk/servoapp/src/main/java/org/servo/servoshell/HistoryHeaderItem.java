package org.servo.servoshell;

public class HistoryHeaderItem implements HistoryItem {
    private final String headerText;
    
    public HistoryHeaderItem(String headerText) {
        this.headerText = headerText;
    }
    
    public String getHeaderText() {
        return headerText;
    }
    
    @Override
    public int getType() {
        return TYPE_HEADER;
    }
}
