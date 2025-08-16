// Function to populate the fake mail client database with folders,
// conversations, and messages.
const populateFakeMailClientDatabase = async (db) => {
  // Creates object stores for folders, conversations, and messageBodies.
  const foldersStore = db.createObjectStore(folderStoreName, {keyPath: 'id'});

  // A conversation can exist in multiple folders, so (folderId, id) is used as
  // the primary key to ensure uniqueness within each folder.
  const conversationsStore = db.createObjectStore(
      conversationStoreName, {keyPath: ['folderId', 'id']});

  // Creates message store and all related indexes
  const messagesStore = db.createObjectStore(messageStoreName, {keyPath: 'id'});

  // Adds an index for sender address to optimize message lookups by sender.
  messagesStore.createIndex('from', 'metaData.sender', {unique: false});

  // Adds indexes for To and CC addresses to optimize message lookups. Using
  // multiEntry: true because 'metaData.to' and 'metaData.cc' are arrays of
  // addresses. This allows each email address in these arrays to be indexed
  // individually, enabling efficient querying for messages sent to or CC'd to
  // specific addresses.
  messagesStore.createIndex(
      'to', 'metaData.to', {unique: false, multiEntry: true});
  messagesStore.createIndex(
      'cc', 'metaData.cc', {unique: false, multiEntry: true});

  // Adds a timestamp index to enable chronological sorting of messages.
  messagesStore.createIndex('sortTime', 'metaData.timestamp', {unique: false});

  // Adds an index to quickly access flagged messages.
  messagesStore.createIndex(
      'flagStatus', 'metaData.isFlagged', {unique: false});

  // Caches all folders, conversations, and messages created during database
  // population. These in-memory arrays are useful for post-initialization
  // validation, debugging, or further operations without reading from the
  // database again.
  const foldersInMemory = [];
  const conversationsInMemory = [];
  const messagesInMemory = [];

  // Populates each folder with conversations and messages.
  for (let folderIndex = 0; folderIndex < folderData.length; folderIndex++) {
    const folder = folderData[folderIndex];
    const folderId = folder.folderId;

    // Computes the total message count by summing messages per conversation.
    const totalMessageCount = folder.conversations.reduce(
        (sum, conversation) => sum +
            (conversation.messagesPerConversation *
             conversation.conversationCount),
        0);

    const folderDetails = {
      id: folderId,
      name: folder.folderName,
      unreadCount: 0,
      totalMessageCount: totalMessageCount,
    };

    let folderUnreadCount = 0;

    // Loops through each conversation group within the folder.
    for (let convGroupIndex = 0; convGroupIndex < folder.conversations.length;
         convGroupIndex++) {
      const conversationGroup = folder.conversations[convGroupIndex];

      // Determines the number of digits needed for zero-padding conversation
      // indices, ensuring consistent ID length and lexicographical sorting of
      // conversation IDs.
      const convPadLength =
          conversationGroup.conversationCount.toString().length;

      // Determines the number of digits needed for zero-padding message
      // indices, ensuring consistent ID length and lexicographical sorting of
      // message IDs.
      const msgPadLength =
          conversationGroup.messagesPerConversation.toString().length;

      // Loops through each conversation in the conversation group.
      for (let convIndex = 0; convIndex < conversationGroup.conversationCount;
           convIndex++) {
        const paddedConvIndex =
            convIndex.toString().padStart(convPadLength, '0');
        const conversationId =
            `conv-${folderIndex}-${convGroupIndex}-${paddedConvIndex}`;
        const messageCount = conversationGroup.messagesPerConversation;

        // Tracks unread messages in this conversation.
        let conversationUnreadCount = 0;

        // Creates messages for the conversation.
        for (let msgIndex = 0; msgIndex < messageCount; msgIndex++) {
          const paddedMsgIndex =
              msgIndex.toString().padStart(msgPadLength, '0');
          const messageId = `msg-${folderIndex}-${convGroupIndex}-${
              paddedConvIndex}-${paddedMsgIndex}`;
          const isRead =
              msgIndex >= Math.floor(messageCount * unreadRate) ? 1 : 0;

          if (!isRead) {
            conversationUnreadCount++;
            folderUnreadCount++;
          }

          const isFlagged = msgIndex % 2;
          const hasAttachments = msgIndex % 2;
          const isImportant = msgIndex % 2;
          const bodyText = `This is the body of message ${msgIndex}`;
          const size = new TextEncoder().encode(bodyText).length;

          const toRecipients = [
            `recipient-${folderIndex}-${convGroupIndex}-${convIndex}-${
                msgIndex}-1@example.com`,
            `recipient-${folderIndex}-${convGroupIndex}-${convIndex}-${
                msgIndex}-2@example.com`
          ];

          const ccRecipients = [
            `cc${folderIndex}-${convGroupIndex}-${convIndex}-${
                msgIndex}-1@example.com`,
            `cc${folderIndex}-${convGroupIndex}-${convIndex}-${
                msgIndex}-2@example.com`
          ];

          const message = {
            id: messageId,
            parentFolderId: folderId,
            conversationId: conversationId,
            metaData: {
              isRead,
              isFlagged,
              isImportant,
              hasAttachments,
              size,
              to: toRecipients,
              cc: ccRecipients,
              sender: `sender${folderIndex}-${convGroupIndex}-${convIndex}-${
                  msgIndex}@example.com`,
              subject: `Subject of message ${msgIndex}`,
              timestamp: new Date().toISOString(),
            },
            body: bodyText,
          };

          // Adds each message to the store directly.
          messagesStore.add(message);
          messagesInMemory.push(message);
        }

        const conversationData = {
          id: conversationId,
          folderId,
          messageIds: Array.from(
              {length: messageCount},
              (_, msgIndex) =>
                  `msg-${folderIndex}-${convGroupIndex}-${paddedConvIndex}-${
                      msgIndex.toString().padStart(msgPadLength, '0')}`),
          unreadCount: conversationUnreadCount,
        };

        conversationsStore.add(conversationData);
        conversationsInMemory.push(conversationData);
      }
    }

    folderDetails.unreadCount = folderUnreadCount;

    // Adds the folder to the store.
    foldersStore.add(folderDetails);
    foldersInMemory.push(folderDetails);
  }

  return {foldersInMemory, conversationsInMemory, messagesInMemory};
};
