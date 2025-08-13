// Populates the fake sync DB with changes to simulate server-side updates.
const populateFakeMailClientSyncDatabase = async (db) => {
  await populateFakeServerChanges(db);
};

/**
 * Generates simulated server-side changes: deletes, updates, and adds.
 * Breaks down the process into smaller helper functions for clarity.
 * @param {IDBDatabase} db - The IndexedDB instance.
 */
const populateFakeServerChanges = async (db) => {
  const {conversationsInMemory} = await populateFakeMailClientDatabase(db);

  let updatedConversations = [];
  let deletedConversations = [];
  const fakeAddedMessageMap = {};
  let newConvIdNum = 0;

  const fakeServerChanges = [];
  const maxChangesReturned = syncConfig.maxChangesReturned;

  let convIndex = 0;
  while (convIndex < conversationsInMemory.length) {
    const conversationsToDelete = conversationsInMemory.slice(
        convIndex, convIndex + syncConfig.conversationsToDelete);
    deletedConversations.push(
        ...createDeletedConversationsBatch(conversationsToDelete));

    convIndex += syncConfig.conversationsToDelete;

    const conversationsToModify = conversationsInMemory.slice(
        convIndex, convIndex + syncConfig.conversationsToUpdateMessageCount);

    updatedConversations.push(...createUpdatedConversationsBatch(
        conversationsToModify, fakeAddedMessageMap));

    convIndex += syncConfig.conversationsToUpdateMessageCount;

    const newConversationsBatch =
        createNewConversationsBatch(++newConvIdNum, fakeAddedMessageMap);
    updatedConversations.push(...newConversationsBatch);

    // Batch changes respecting maxChangesReturned limit.
    if ((deletedConversations.length + updatedConversations.length) >
        maxChangesReturned) {
      const {deletedBatch, updatedBatch, remainingDeleted, remainingUpdated} =
          splitBatches(
              deletedConversations, updatedConversations, maxChangesReturned);

      fakeServerChanges.push({
        deletedConversations: deletedBatch,
        updatedConversations: updatedBatch,
      });

      deletedConversations = remainingDeleted;
      updatedConversations = remainingUpdated;
    }
  }

  // Add any remaining changes if they exist.
  if (deletedConversations.length || updatedConversations.length) {
    fakeServerChanges.push({
      deletedConversations,
      updatedConversations,
    });
  }

  // Store batched server changes and new message contents for later retrieval.
  self.parent.fakeConvChanges = fakeServerChanges;
  self.parent.fakeAddedMessageMap = fakeAddedMessageMap;
};

/**
 * Creates an array of deleted conversation objects from a slice of
 * conversations.
 * @param {Array} conversationsToDelete - Array of conversations to delete.
 * @returns {Array} - Array of objects representing deleted conversations.
 */
const createDeletedConversationsBatch = (conversationsToDelete) => {
  return conversationsToDelete.map(
      conversation => ({
        // Pair folderId and conversation.id to uniquely identify conversations
        // across multiple folders.
        id: [conversation.folderId, conversation.id],
        messageIds: conversation.messageIds || [],
        unreadCount: conversation.unreadCount,
      }));
};

/**
 * Creates updated conversations by appending new messages and generating
 * corresponding fake messages.
 * @param {Array} conversationsToModify - Conversations to update.
 * @param {Object} fakeAddedMessageMap - Map to store newly created fake
 *     messages.
 * @returns {Array} - Array of updated conversation objects.
 */
const createUpdatedConversationsBatch =
    (conversationsToModify, fakeAddedMessageMap) => {
      const updatedConversations = [];

      // Determine the padding length for new message indexes,
      // e.g. if messagesToAppendPerConversation = 10, length is 2 (because
      // "10".length = 2) So msgIndex 0 -> "00", msgIndex 1 -> "01", ... to keep
      // IDs consistent length.
      const newMsgPadLength =
          syncConfig.messagesToAppendPerConversation.toString().length;

      conversationsToModify.forEach(conversation => {
        const updatedMessageIds = [...conversation.messageIds];
        const addedMessageIds = [];

        for (let msgIndex = 0;
             msgIndex < syncConfig.messagesToAppendPerConversation;
             msgIndex++) {
          // Pad the msgIndex with leading zeros to maintain uniform length
          // For example, if msgIndex = 3 and pad length = 2, paddedNewMsgIndex
          // = "03"
          const paddedNewMsgIndex =
              msgIndex.toString().padStart(newMsgPadLength, '0');

          // Create message ID using padded index.
          // Example: if conversation.id = "conv-123", replaced with
          // "msg-123-new-03"
          addedMessageIds.push(
              `${conversation.id.replace('conv-', 'msg-')}-new-${
                  paddedNewMsgIndex}`);
        }

        // Store the generated fake messages for this conversation
        fakeAddedMessageMap[conversation.id] = createFakeMessages(
            conversation.folderId, conversation.id, addedMessageIds);

        // Append the new message IDs to the existing ones
        updatedMessageIds.push(...addedMessageIds);

        updatedConversations.push({
          id: conversation.id,
          folderId: conversation.folderId,
          unreadCount: conversation.unreadCount +
              syncConfig.messagesToAppendPerConversation,
          messageIds: updatedMessageIds,
        });
      });
      return updatedConversations;
    };

/**
 * Creates a batch of new conversations along with their fake messages.
 * @param {number} startingConvIdNum - The starting index for new conversation
 *     IDs.
 * @param {Object} fakeAddedMessageMap - Map to store newly created fake
 *     messages.
 * @returns {Array} - Array of new conversation objects.
 */
const createNewConversationsBatch =
    (startingConvIdNum, fakeAddedMessageMap) => {
      const newConversations = [];

      // Determine padding lengths to keep IDs consistent in length
      // For example, conversationsToAdd = 10 -> pad length = 2 (to pad "1" as
      // "01")
      const convPadLength = syncConfig.conversationsToAdd.toString().length;
      const msgPadLength =
          syncConfig.messagesPerNewConversation.toString().length;

      for (let addIndex = 0; addIndex < syncConfig.conversationsToAdd;
           addIndex++) {
        // Pad the conversation index
        // Example: startingConvIdNum = 1, addIndex = 2, paddedConvIndex = "03"
        // (if padLength=2)
        const paddedConvIndex = (startingConvIdNum + addIndex)
                                    .toString()
                                    .padStart(convPadLength, '0');

        // New conversation ID example: "conv-1-1-new-03"
        const newConvId = `conv-1-1-new-${paddedConvIndex}`;
        const messageIds = [];

        for (let msgIndex = 0; msgIndex < syncConfig.messagesPerNewConversation;
             msgIndex++) {
          // Pad the message index similarly, e.g. "00", "01", "02", etc.
          const paddedMsgIndex =
              msgIndex.toString().padStart(msgPadLength, '0');

          // Message ID example: "msg-1-1-new-03-00"
          messageIds.push(`msg-1-1-new-${paddedConvIndex}-${paddedMsgIndex}`);
        }

        // Store fake messages for the new conversation
        fakeAddedMessageMap[newConvId] =
            createFakeMessages(targetFolderId, newConvId, messageIds);

        newConversations.push({
          id: newConvId,
          folderId: targetFolderId,
          unreadCount: syncConfig.messagesPerNewConversation,
          messageIds,
        });
      }
      return newConversations;
    };

/**
 * Splits deleted and updated conversations into batches that fit within
 * maxChangesReturned.
 * @param {Array} deletedConversations - Array of deleted conversations.
 * @param {Array} updatedConversations - Array of updated conversations.
 * @param {number} maxChangesReturned - Maximum allowed changes per batch.
 * @returns {Object} - Object containing batched arrays and remaining arrays.
 */
const splitBatches =
    (deletedConversations, updatedConversations, maxChangesReturned) => {
      const deletedSize =
          Math.min(maxChangesReturned, deletedConversations.length);
      const deletedBatch = deletedConversations.slice(0, deletedSize);

      const updatedSize = Math.min(
          maxChangesReturned - deletedSize, updatedConversations.length);
      const updatedBatch = updatedConversations.slice(0, updatedSize);

      const remainingDeleted = deletedConversations.slice(deletedSize);
      const remainingUpdated = updatedConversations.slice(updatedSize);

      return {
        deletedBatch,
        updatedBatch,
        remainingDeleted,
        remainingUpdated,
      };
    };

// Creates fake message content using given IDs, associated to a specific folder
// and conversation.
const createFakeMessages = (folderId, conversationId, messageIds) => {
  return messageIds.map(
      (id, index) => ({
        id,
        conversationId: conversationId,
        parentFolderId: folderId,
        subject: `New message ${index} for conversation ${conversationId}`,
        body: `New message body ${index} for conversation ${conversationId}`,
        metaData: {
          from: 'sender@example.com',
          to: ['recipient@example.com'],
          cc: ['cc1@example.com'],
          bcc: [],
          date: new Date().toISOString(),
          isRead: 0,
        },
      }));
};
