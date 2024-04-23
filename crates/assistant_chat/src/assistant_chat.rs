#![allow(unused, dead_code)]
//! # UI – Chat List

use std::sync::Arc;

use client::{User, UserStore};
use editor::*;
use gpui::*;
use indoc::indoc;
use language::{language_settings::SoftWrap, Buffer, LanguageRegistry, ToOffset as _};
use rich_text::RichText;
use settings::Settings;
use theme::ThemeSettings;
use ui::*;
use workspace::Workspace;

// TODO next
//
// ## Chat Messsage
// - [ ] render ChatMessages as RichText
//  - [x] get the LanguageRegistry on to ChatList
//  - [x] either pass a RichText to ChatMessage, or create the RichText from string in ChatMessage
//  - [ ] split ChatMessage into new_user and new_assistant
// - [ ] Build rendering for ChatContext
// - [ ] Hook up message collapsing
//
// ## Chat List
// - render a list of chat messages
//
// ## Composer
// - build out composer static UI
// - add editor
// - add button on_click actions for Send and Quote Selection
// - add model switcher

pub struct ChatList {
    workspace: WeakView<Workspace>,
    languages: Arc<LanguageRegistry>,
    user_store: Model<UserStore>,
    messages: Vec<(ChatRole, Arc<str>)>,
    composer: View<Composer>,
}

impl ChatList {
    pub fn new(
        workspace: WeakView<Workspace>,
        user_store: Model<UserStore>,
        cx: &mut ViewContext<Self>,
    ) -> Result<ChatList> {
        let message_1 = indoc! {r#"I want to write a python script to output all unique @{name} instancees across all highlight.scm files in my project. For example, @variable, @function, @class, etc.

        The script should output all unique instances of @{name} in the highlight.scm files, and addionally add a count to each.

        For example:

        ```
        @variable (100)
        @function (50)
        ```

        The list should be sorted in alphabetical order."#};
        let message_2 = indoc! {r#"Given your requirements, here's a Python script that will search for all unique instances of `@{name}` in `highlight.scm` files both in the core languages path and the extensions path. It will also count the occurrences of each instance and organize the output as specified:

            ```python
            def parse_arguments():
                parser = argparse.ArgumentParser(description='Analyze highlight.scm files for unique instances and their languages.')
                parser.add_argument('-v', '--verbose', action='store_true', help='Include a list of languages for each tag.')
                return parser.parse_args()

            def find_highlight_files(root_dir):
                for path in Path(root_dir).rglob('highlights.scm'):
                    yield path

            def count_instances(files):
                instances: defaultdict[list[Any], dict[str, Any]] = defaultdict(lambda: {'count': 0, 'languages': set()})
                for file_path in files:
                    language = file_path.parent.name
                    with open(file_path, "r") as file:
                        text = file.read()
                        matches = pattern.findall(text)
                        for match in matches:
                            instances[match]['count'] += 1
                            instances[match]['languages'].add(language)
                return instances

            ```

            "#};

        let static_messages = vec![
            (ChatRole::User, message_1.into()),
            (ChatRole::Assistant, message_2.into()),
            (ChatRole::User, message_1.into()),
            (ChatRole::Assistant, message_2.into()),
            (ChatRole::User, message_1.into()),
            (ChatRole::Assistant, message_2.into()),
            (ChatRole::User, message_1.into()),
            (ChatRole::Assistant, message_2.into()),
        ];

        let composer = cx.new_view(|_| Composer {});

        let workspace_handle = workspace.clone();

        workspace.update(cx, |workspace, cx| Self {
            user_store,
            languages: workspace.app_state().languages.clone(),
            workspace: workspace_handle,
            messages: static_messages,
            composer,
        })
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
        let Some(current_user) = self.current_user(cx) else {
            return div().id("empty").child("Loading...");
        };

        let messages = self.messages.iter().map(|(role, message)| {
            let rich_text =
                rich_text::render_rich_text(message.to_string(), &[], &self.languages, None);

            ChatMessage::new(*role, current_user.clone(), rich_text)
        });

        div()
            .id("chat-list")
            .size_full()
            .overflow_y_scroll()
            .child(v_flex().max_w(rems(40.0)).gap_2().p_4().children(messages))
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
    message: RichText,
    collapsed: bool,
}

impl ChatMessage {
    pub fn new(role: ChatRole, player: Arc<User>, message: RichText) -> ChatMessage {
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
        let content = div()
            .overflow_hidden()
            .w_full()
            .child(self.message.element("message".into(), cx));

        v_flex()
            .child(header)
            .child(h_flex().gap_2().child(collapse_handle).child(content))
    }
}

pub struct ChatInlineNotice {}

// === Chat Header ===

#[derive(Debug, Clone, Copy)]
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
