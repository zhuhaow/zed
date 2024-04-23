//! # UI – Chat List

use std::sync::Arc;

use client::{User, UserStore};
use editor::*;
use gpui::*;
use settings::Settings;
use theme::ThemeSettings;
use ui::*;

pub struct ChatList {
    user_store: Model<UserStore>,
}

impl ChatList {
    pub fn new(user_store: Model<UserStore>, cx: ViewContext<Self>) -> ChatList {
        ChatList { user_store }
    }

    pub fn current_user(&self, cx: ViewContext<Self>) -> Option<Arc<User>> {
        self.user_store.read(&cx).current_user()
    }

    pub fn assistant_user() -> User {
        User {
            id: 99999,
            github_login: "Assistant".into(),
            avatar_uri: "https://avatars.githubusercontent.com/u/1714999?v=4".into(),
        }
    }

    pub fn static_user() -> User {
        User {
            id: 99998,
            github_login: "iamnbutler".into(),
            avatar_uri: "https://avatars.githubusercontent.com/u/1714999?v=4".into(),
        }
    }
}

pub struct Composer {}

pub struct ChatMessage {
    role: ChatRole,
    player: User,
    collapsed: bool,
}

impl ChatMessage {
    pub fn new(role: ChatRole, player: User) -> ChatMessage {
        ChatMessage {
            role,
            player,
            collapsed: false,
        }
    }

    pub fn collapsed(&mut self, collapsed: bool) -> &mut Self {
        self.collapsed = collapsed;
        self
    }
}

impl RenderOnce for ChatMessage {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let header = ChatHeader::new(self.role, self.player);
        let content = div().child("Hello, world!");

        v_flex().child(header).child(content)
    }
}

pub struct ChatInlineNotice {}

// === Chat Header ===

pub enum ChatRole {
    User,
    Assistant,
    Notice,
}

pub enum ChatContextType {
    Code,
    Diagnostic,
    Docs,
}

pub struct ChatContext {
    context_type: ChatContextType,
    content: String,
}

struct ChatContexts {
    contexts: Vec<ChatContext>,
}

#[derive(IntoElement)]
pub struct ChatHeader {
    role: ChatRole,
    player: User,
    sent_at: String,
    contexts: Vec<String>,
}

impl ChatHeader {
    pub fn new(role: ChatRole, player: User) -> ChatHeader {
        // use something real
        let sent_at = "now".to_string();

        ChatHeader {
            role,
            player,
            sent_at,
            contexts: Vec::new(),
        }
    }

    pub fn role(&mut self, role: ChatRole) -> &mut Self {
        self.role = role;
        self
    }

    pub fn player(&mut self, player: User) -> &mut Self {
        self.player = player;
        self
    }

    pub fn sent_at(&mut self, sent_at: String) -> &mut Self {
        self.sent_at = sent_at;
        self
    }

    pub fn contexts(&mut self, contexts: Vec<String>) -> &mut Self {
        self.contexts = contexts;
        self
    }
}

impl RenderOnce for ChatHeader {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let player_avatar = Avatar::new(self.player.avatar_uri);
        let player_name = Label::new(self.player.github_login).color(Color::Default);
        let sent_at = Label::new(self.sent_at).color(Color::Muted);

        h_flex()
            .justify_between()
            .child(
                h_flex()
                    .gap_1()
                    .child(player_avatar)
                    .child(player_name)
                    .child(sent_at),
            )
            .child(div().when(self.contexts.len() > 0, |this| {
                this.child(Label::new(self.contexts.len().to_string()).color(Color::Muted))
            }))
    }
}
