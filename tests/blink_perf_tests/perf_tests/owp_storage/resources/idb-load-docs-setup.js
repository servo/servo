// Our database schema:
// Users
// * key - string
// * value - dictionary w/ strings and an array
// * 1 entry
// DocumentLocks
// * key - array w/ one string item
// * value - dictionary w/ string, number, and array of string
// * 469 entries
// Documents
// * key - string
// * value - dictionary, w/ nested dictionaries, strings, numbers, arrays
//   (one of which has lots of items - numbers, strings, further arrays)
// * 730 entries
// PendingQueues
// * key - string
// * value - dictionary w/ empty array, strings, numbers, bools, undefineds
// * 730 entries
// DocumentCommands
// * key - array
// * value - everything! large.
// * 2000 entries - we only do 20 to make things not too long for startup.
// PendingQueueCommands
// * empty
// SyncObjects
// * key - array of strings
// * value - dictionary w/ dictionaries, keypath, and timestamp
// * 55 entries
// FontMetadata
// * key - string of font name
// * value - dictionary of arrays of dictionaries. strings, numbers, etc
let populateFakeDocsDatabase = function(db) {

  function randomAlphaNum(length) {
    const chars = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-';
    let result = '';
    for (let i = length; i > 0; --i)
      result += chars[Math.floor(Math.random() * chars.length)];
    return result;
  }

  let otherDocsIds = [];
  for (let i = 0; i < 729; i++) {
    otherDocsIds.push(randomAlphaNum(44));
  }

  let populateUser = function() {
    let users = db.createObjectStore("Users", {
      keyPath: "id"
    });
    users.put(UsersValue);
  }

  let populateDocumentLocks = function() {
    let documentLocks = db.createObjectStore("DocumentLocks", {
      keyPath: "dlKey"
    });
    documentLocks.put(DocumentLocksValue);
    DocumentLocksValue.dlKey = [docId2];
    documentLocks.put(DocumentLocksValue);

    for (let other_doc_id of otherDocsIds) {
      DocumentLocksValue.id = other_doc_id;
      documentLocks.put(DocumentLocksValue);
    }
  }

  let populateDocuments = function() {
    // first put our 2 docs, then copy the rest.
    let documents = db.createObjectStore("Documents", {
      keyPath: "id"
    });
    documents.put(Documents1Value);
    documents.put(Documents2Value);

    for (let other_doc_id of otherDocsIds) {
      Documents2Value.id = other_doc_id;
      documents.put(Documents2Value);
    }
  }

  let populateDocumentCommands = function() {
    let commands = db.createObjectStore("DocumentCommands", {
      keyPath: "dcKey"
    });
    commands.put(t17_DocumentCommandsValue);

    for (let i = 0; i < 50; i++) {
      t17_DocumentCommandsValue.dcKey[0] = randomAlphaNum(44);
      commands.put(t17_DocumentCommandsValue);
    }
  }

  let populatePendingQueues = function() {
    // first put our 2 docs, then copy the rest.
    let queues = db.createObjectStore("PendingQueues", {
      keyPath: "docId"
    });
    queues.put(PendingQueues1Value);
    queues.put(PendingQueues2Value);

    for (let other_doc_id of otherDocsIds) {
      PendingQueues2Value.id = other_doc_id;
      queues.put(PendingQueues2Value);
    }
  }

  let createPendingQueueCommands = function() {
    db.createObjectStore("PendingQueueCommands");
  }

  let populateSyncObjects = function() {
    let sync_objects = db.createObjectStore("SyncObjects", {
      keyPath: "keyPath"
    });
    sync_objects.put(SyncObjects1Value);
    sync_objects.put(SyncObjects2Value);
    sync_objects.put(SyncObjects3Value);
    sync_objects.put(SyncObjects4Value);
    for (let i = 0; i < 51; ++i) {
      SyncObjects1Value.keyPath[2] = randomAlphaNum(10);
      sync_objects.put(SyncObjects4Value);
    }
  }

  let populateFontMetadata = function() {
    let fonts = db.createObjectStore("FontMetadata", {
      keyPath: "fontFamily"
    });
    fonts.put(FontMetadata1Value);
    fonts.put(FontMetadata2Value);

    for (let i = 0; i < 148; ++i) {
      FontMetadata1Value.fontFamily = randomAlphaNum(10);
      fonts.put(FontMetadata1Value);
    }
  }

  let createBlobMetadata = function() {
    db.createObjectStore("BlobMetadata", {
      keyPath: ["d", "p"]
    });
  }

  populateUser();
  populateDocumentLocks();
  populateDocuments();
  populateDocumentCommands();
  populatePendingQueues();
  createPendingQueueCommands();
  populateSyncObjects();
  populateFontMetadata();
  createBlobMetadata();
}
