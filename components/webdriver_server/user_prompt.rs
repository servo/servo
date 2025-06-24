use std::collections::HashMap;

type UserPrompt = String;

struct UserPromptConfig {
    handler: String,
    notify: bool,
}

type UserPromptHandler = HashMap<UserPrompt, UserPromptConfig>;