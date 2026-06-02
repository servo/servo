// This configuration file defines the settings and parameters
// for the IndexedDB database used in the mail client. It specifies
// the database name, object store names, folder data configuration,
// and the batch size for operations. This file is crucial for initializing
// and managing the database schema and operations for the mail client,
// ensuring a consistent structure for folders, conversations, and messages.

// The name of the IndexedDB database.
// This value is used when creating or opening the database.
const databaseName = 'MailClientDB';

// The name of the object store used for storing folder information.
// A folder contains metadata about email messages and conversations.
const folderStoreName = 'folders';

// The name of the object store used for storing conversation information.
// A conversation contains a collection of messages and tracks unread counts.
const conversationStoreName = 'conversations';

// The name of the object store used for storing message bodies.
// A message represents a single email and contains detailed information
// such as sender, recipients, subject, and message body.
const messageStoreName = 'messageBodies';

// Configuration for folder data, which defines how many conversations
// and messages a single folder contains. This data is used to initialize
// the folder structure in the database.
const folderData = [{
  // Name of the folder
  folderName: 'Folder 1',
  // Unique ID for the folder
  folderId: 'folder-1',
  conversations: [
    // 100 conversations, each with 5 messages
    {messagesPerConversation: 5, conversationCount: 100},

    // 100 conversations, each with 15 messages
    {messagesPerConversation: 15, conversationCount: 100},

    // 50 conversations, each with 1 message
    {messagesPerConversation: 1, conversationCount: 50},

    // 50 conversations, each with 19 messages
    {messagesPerConversation: 19, conversationCount: 50}
  ]
}];

// The targetFolderId represents the ID of the folder to mark as read.
// It dynamically retrieves the folderId from the folderData configuration
// to ensure flexibility if the folder structure changes.
const targetFolderId = folderData[0].folderId;

// Defines the number of conversations to process in each batch for efficient
// handling of large datasets. This value helps paginate through conversations
// in a folder incrementally, reducing memory usage and improving performance
// by processing smaller chunks of data at a time, instead of fetching all
// conversations at once.
const batchSize = 50;

// Defines the unread message rate as a percentage, which determines the
// proportion of messages marked as unread. In this case, 70% of messages
// are marked as unread.
const unreadRate = 0.7;

// Sync configuration for the mail client.
// This configuration defines how conversations and messages are processed
// during synchronization. Each parameter is explained with an example to help
// visualize the behavior.
const syncConfig = {
  // Number of conversations to delete in each batch.
  // Example: If `conversationsToDelete` is 5, the first 5 conversations (e.g.,
  // conv-1-1-1 to conv-1-1-5)
  // will be removed from the database during each sync batch.
  conversationsToDelete: 5,

  // Number of conversations to update by appending new messages.
  // Example: If `conversationsToUpdateMessageCount` is 10, the next 10
  // conversations (e.g., conv-1-1-6 to conv-1-1-15) will have messages
  // appended to them.
  conversationsToUpdateMessageCount: 10,

  // Number of messages to append to each updated conversation.
  // Example: If `messagesToAppendPerConversation` is 5, each of the updated
  // conversations will have 5 new messages appended with shared subject
  // prefixes.
  messagesToAppendPerConversation: 5,

  // Number of new conversations to add in each batch.
  // Example: If `conversationsToAdd` is 5, 5 new conversations (e.g.,
  // conv-1-1-new-1 to conv-1-1-new-5)
  // will be added to the database after the other operations are complete.
  conversationsToAdd: 5,

  // Number of messages to add in each new conversation.
  // Example: If `messagesPerNewConversation` is 2, each new conversation (e.g.,
  // conv-1-1-new-1 to conv-1-1-new-5)
  // will have 2 messages added with unique subjects and bodies.
  messagesPerNewConversation: 2,

  // Maximum number of changes returned in a single operation.
  // This limits the number of updates, messages, or records returned to avoid
  // overwhelming the client or server and to ensure consistent performance.
  maxChangesReturned: 50,
};
