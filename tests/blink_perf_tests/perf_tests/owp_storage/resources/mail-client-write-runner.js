const openRequest = window.indexedDB.open(databaseName);

// Event handler for successful database connection
openRequest.onsuccess = async () => {
  const db = openRequest.result;
  try {
    // We use reportDone() to indicate to the Perf test runner that the page
    // load time measurement should stop after the marking of all messages as
    // read.
    await markFolderAsRead(db, targetFolderId);
    reportDone();
  } catch (error) {
    reportError('Error marking folder as read: ', error);
  }
};

// Event handler for database open error
openRequest.onerror = (event) => {
  reportError('Error opening database: ', event.target.error);
};

/**
 * Marks all conversations within a folder as read.
 *
 * This function processes conversations in a folder incrementally in batches to
 * avoid fetching all conversations at once. It uses a compound index
 * ('parentFolderId+id') for efficient querying and paginates using the last
 * conversation ID in each batch. The loop terminates when no more conversations
 * are found, ensuring scalability for folders with large datasets.
 *
 * @param {IDBDatabase} db The IndexedDB database instance.
 * @param {string} folderId The ID of the folder to mark as read.
 * @throws {Error} If the folder is not found in the database.
 */
const markFolderAsRead = async (db, folderId) => {
  // Open a transaction to fetch the folder object from the store using
  // folderStoreName.
  const folder = await getIDBValue(
      db.transaction(folderStoreName, 'readonly'), folderStoreName, folderId);

  if (!folder) {
    reportError(`Folder not found: ${folderId}`);
  }

  // Initialize with folderId and empty conversationId for pagination.
  let lastConversationIndex = [folderId, ''];

  // Loop to fetch and process conversations in batches.
  while (true) {
    const convTransaction = db.transaction(conversationStoreName, 'readonly');

    // Defines the key range for pagination, excluding lastConversationIndex.
    const range = IDBKeyRange.lowerBound(lastConversationIndex, true);

    // Fetch a batch of conversation values (with keys) up to the batch size.
    const conversationBatch = await getAllIDBValues(
        convTransaction, conversationStoreName, range, batchSize);

    if (conversationBatch.length === 0) {
      break;
    }

    // Update lastConversationIndex with the last conversation in the batch.
    lastConversationIndex =
        [folderId, conversationBatch[conversationBatch.length - 1].id];

    try {
      await processBatch(db, conversationBatch, folder);
    } catch (error) {
      reportError(`Error processing batch: `, error);
    }
  }
};

/**
 * Processes a batch of conversations and marks them as read.
 *
 * This function performs the actual updates to conversations, messages, and
 * folder unread counts within a single transaction. The changes are applied
 * atomically.
 *
 * @param {IDBDatabase} db The IndexedDB database instance.
 * @param {Array} batch The batch of conversation objects to process.
 * @param {Object} folder The folder object containing unread count.
 */
const processBatch = async (db, batch, folder) => {
  // Open a readwrite transaction for the folderStoreName,
  // conversationStoreName, and messageStoreName.
  const transaction = db.transaction(
      [folderStoreName, conversationStoreName, messageStoreName], 'readwrite');

  const conversationUpdates = [];
  const messageUpdates = [];
  let folderUnreadChange = 0;

  for (const conversation of batch) {
    try {
      for (const messageId of conversation.messageIds) {
        const message =
            await getIDBValue(transaction, messageStoreName, messageId);
        if (message.metaData.isRead !== 1) {
          message.metaData.isRead = 1;
          messageUpdates.push(message);
        }
      }

      // Update the conversation unread count.
      if (conversation.unreadCount !== 0) {
        folderUnreadChange += conversation.unreadCount;
        conversation.unreadCount = 0;
        conversationUpdates.push(conversation);
      }
    } catch (error) {
      reportError(
          `Error processing conversationId ${conversation.id}: `, error);
    }
  }

  // Apply updates to messages
  for (const message of messageUpdates) {
    await putIDBValue(transaction, messageStoreName, message);
  }

  // Apply updates to conversations
  for (const conversation of conversationUpdates) {
    await putIDBValue(transaction, conversationStoreName, conversation);
  }

  // Update the folder unread count
  if (folderUnreadChange !== 0) {
    folder.unreadCount -= folderUnreadChange;
    await putIDBValue(transaction, folderStoreName, folder);
  }

  // Update the read status on the server
  await updateReadStatusOnMailbox(conversationUpdates, messageUpdates, folder);
};

/**
 * Simulates updating the read status on the mailbox.
 *
 * @param {Map} conversationUpdates The map of updated conversations.
 * @param {Array} messageUpdates The array of updated messages.
 * @param {Object} folder The folder object containing unread count.
 */
const updateReadStatusOnMailbox =
    async (conversationUpdates, messageUpdates, folder) => {
  const updateData = {
    folder: {id: folder.id, unreadCount: folder.unreadCount},
    conversations: conversationUpdates.map(
        convo => ({id: convo.id, unreadCount: convo.unreadCount})),
    messages: messageUpdates.map(
        message => ({id: message.id, isRead: message.metaData.isRead}))
  };

  return new Promise((resolve) => {
    setTimeout(() => {
      resolve();
    }, 0);
  });
};

