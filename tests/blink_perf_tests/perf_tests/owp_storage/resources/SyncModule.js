// syncModule.js - Provides synchronization functionality for conversations and
// messages

/**
 * Factory function that creates a sync module for a specific folder
 * @param {IDBDatabase} db - The database connection
 * @param {string} folderId - The ID of the folder to sync
 * @returns {SyncModule} - The sync module
 */
function getSyncModule(db, folderId) {
  return new SyncModule(db, folderId);
}

/**
 * Handles conversations and messages synchronization for a specific folder
 * Processes conversations and messages additions, updates, and deletions during
 * sync
 */
class SyncModule {
  constructor(db, folderId) {
    this.db = db;
    this.folderId = folderId;
    this.batchIndex = 0;

    // Load and parse fake server changes from localStorage once per instance.
    // These simulate the server responses across sync cycles.
    // Cache the list of conversation changes for batched sync simulation.
    this.cachedConvChanges = self.parent.fakeConvChanges;

    // Map of conversationId -> newly added message IDs for each conversation.
    this.addedMessagesMap = self.parent.fakeAddedMessageMap;
  }

  async syncChangesFromServer() {
    // If there are no more batches to process, return null to signal sync
    // complete.
    if (this.batchIndex === this.cachedConvChanges.length) {
      return null;
    }

    // Fetch the current batch and increment the index to point to the next
    // batch.
    const batch = this.cachedConvChanges[this.batchIndex++];

    return batch;
  }

  async saveChangesToStore(response) {
    // Open a transaction that includes all involved stores.
    const transaction = this.db.transaction(
        [conversationStoreName, messageStoreName, folderStoreName],
        'readwrite');

    const conversationStore = transaction.objectStore(conversationStoreName);
    const messageStore = transaction.objectStore(messageStoreName);

    // Tracks net unread count change for the folder during this sync cycle.
    let folderUnreadChange = 0;

    // Apply updated conversations to the database.
    await bulkPutIDBValues(conversationStore, response.updatedConversations);

    // Extract conversation IDs to delete and remove from store.
    const deletedIds = response.deletedConversations.map(c => c.id);
    await bulkDeleteIDBValues(conversationStore, deletedIds);

    // Delete all messages associated with deleted conversations.
    for (const conversation of response.deletedConversations) {
      folderUnreadChange -= conversation.unreadCount;
      await bulkDeleteIDBValues(messageStore, conversation.messageIds);
    }

    // Process each updated conversation in the batch.
    for (const conversation of response.updatedConversations) {
      const newMessages = this.addedMessagesMap[conversation.id];

      // Split message IDs to update vs. newly added.
      const updatedMessageIds = conversation.messageIds;
      // Exclude newMessageIds from updatedMessageIds and get existingMessageIds
      const newMessageIds = newMessages.map(msg => msg.id);
      const existingMessageIds =
          updatedMessageIds.filter(id => !newMessageIds.includes(id));

      const messages = await bulkGetIDBValues(messageStore, existingMessageIds);

      // Refresh timestamps to simulate changes.
      const updatedMessages = messages.map(msg => {
        msg.metaData.timestamp = new Date().toISOString();
        return msg;
      });

      updatedMessages.push(...newMessages);
      await bulkPutIDBValues(messageStore, updatedMessages);

      folderUnreadChange += newMessages.length;
    }

    // Apply net unread count delta to the folder and persist changes.
    const folder =
        await getIDBValue(transaction, folderStoreName, this.folderId);
    folder.unreadCount += folderUnreadChange;
    await putIDBValue(transaction, folderStoreName, folder);
  }
}
