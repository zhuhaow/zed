#![allow(unused, dead_code)]
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
    message_list: Vec<ChatMessage>,
    composer: View<Composer>,
}

impl ChatList {
    pub fn new(cx: &mut ViewContext<Self>, user_store: Model<UserStore>) -> ChatList {
        let composer = cx.new_view(|_| Composer {});

        ChatList {
            user_store,
            message_list: Vec::new(),
            composer,
        }
    }

    pub fn current_user(&self, cx: &ViewContext<Self>) -> Option<Arc<User>> {
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

impl Render for ChatList {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let current_user = self.current_user(cx);
        let message = r#"# Zed

[![CI](https://github.com/zed-industries/zed/actions/workflows/ci.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/ci.yml)

Welcome to Zed, a high-performance, multiplayer code editor from the creators of [Atom](https://github.com/atom/atom) and [Tree-sitter](https://github.com/tree-sitter/tree-sitter)."#;

        if current_user.is_some() {
            div().id("chat-list").size_full().overflow_y_scroll().child(
                v_flex()
                    .max_w(rems(40.0))
                    .gap_2()
                    .p_4()
                    .child(ChatMessage::new(
                        ChatRole::User,
                        current_user.expect("somehow user is not logged in"),
                        message.to_string(),
                    )),
            )
        } else {
            div().id("empty").child("Loading...")
        }
    }
}

pub struct Composer {}

impl Render for Composer {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(IntoElement)]
pub struct ChatMessage {
    role: ChatRole,
    player: Arc<User>,
    // This will likely be RichText
    message: String,
    collapsed: bool,
}

impl ChatMessage {
    pub fn new(role: ChatRole, player: Arc<User>, message: String) -> ChatMessage {
        ChatMessage {
            role,
            player,
            message,
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
        let collapse_handle = h_flex()
            .flex_none()
            .justify_center()
            .w_5()
            .h_full()
            .child(div().w_px().h_full().bg(cx.theme().colors().border));
        let content = div().overflow_hidden().w_full().child(self.message);

        v_flex()
            .child(header)
            .child(h_flex().gap_2().child(collapse_handle).child(content))
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
    player: Arc<User>,
    sent_at: String,
    contexts: Vec<String>,
}

impl ChatHeader {
    pub fn new(role: ChatRole, player: Arc<User>) -> ChatHeader {
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

    pub fn player(&mut self, player: Arc<User>) -> &mut Self {
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
        let player_avatar = Avatar::new(self.player.avatar_uri.clone());
        let player_name = Label::new(self.player.github_login.clone()).color(Color::Default);
        let sent_at = Label::new(self.sent_at).color(Color::Muted);

        h_flex()
            .justify_between()
            .child(
                h_flex()
                    .gap_2()
                    .child(player_avatar)
                    .child(player_name)
                    .child(sent_at),
            )
            .child(div().when(self.contexts.len() > 0, |this| {
                this.child(Label::new(self.contexts.len().to_string()).color(Color::Muted))
                // this.child(Button::new("View Contexts")))
            }))
    }
}
