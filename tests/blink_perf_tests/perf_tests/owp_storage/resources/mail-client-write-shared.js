// Schema for the `messageBodies` object store.
// Represents a single message body in the database.
// Example:
// {
//   id: 'msg-1-2-3-4',            // Unique message ID:
//                                   `msg-{folderIndex}-{convGroupIndex}-{convIndex}-{msgIndex}`.
//   parentFolderId: 'folder-1',   // Parent folder ID for reverse lookup.
//   conversationId: 'conv-1-2-3', // Conversation ID for reverse lookup.
//   metaData: {                   // Message properties.
//     isRead: 0,                  // Read status: 0 (unread), 1 (read).
//                                    IndexedDB doesn't support boolean
//                                    indexing.
//     isFlagged: 0,               // Flag status: 0 (unflagged), 1 (flagged).
//     isImportant: 0,             // Importance: 0 (not important),
//                                    1(important).
//     hasAttachments: 1,          // Attachment presence: 0 (none),
//                                    1(has attachments).
//     size: 1024,                 // Size in bytes.
//     to: [                       // Primary recipients.
//       'recipient1@example.com',
//       'recipient2@example.com'
//     ],
//     cc: [                                // CC recipients.
//       'cc1@example.com',
//       'cc2@example.com'
//     ],
//     sender: 'sender@example.com',        // Sender's email.
//     subject: 'Message subject.',         // Subject line.
//     timestamp: '2024-10-14T11:48:26.123Z'// Creation timestamp.
//   },
//   body: 'Message body.'                  // Message content.
// }

// Schema for the `folders` object store.
// Represents a folder containing messages and conversations.
// Example:
// {
//   id: 'folder-1',                        // Unique folder ID.
//   name: 'Inbox',                         // Folder name.
//   unreadCount: 5,                        // Count of unread messages.
//   totalMessageCount: 10                  // Total messages in the folder.
// }

// Schema for the `conversations` object store.
// Represents a conversation containing messages.
// Example:
// {
//   id: 'conv-1-2-3',               // Unique conversation ID:
//                                      `conv-{folderIndex}-{convGroupIndex}-{convIndex}`.
//   folderId: 'folder-1',           // ID of the containing folder.
//   messageIds: ['msg-1', 'msg-3'], // List of message IDs in the conversation.
//   unreadCount: 1                  // Count of unread messages in the
//   conversation.
// }

// Note: These examples illustrate the schema for the object stores.
// They exclude implementation details and are for reference only.
