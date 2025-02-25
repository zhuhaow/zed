//! Module for managing breakpoints in a project.
//!
//! Breakpoints are separate from a session because they're not associated with any particular debug session. They can also be set up without a session running.
use crate::{
    buffer_store::{BufferStore, BufferStoreEvent},
    BufferId, ProjectItem as _, ProjectPath, WorktreeStore,
};
use anyhow::{Context as _, Result};
use collections::{BTreeMap, HashMap, HashSet};
use dap::{debugger_settings::DebuggerSettings, SourceBreakpoint};
use gpui::{App, AsyncApp, Context, Entity, EventEmitter};
use language::{
    proto::{deserialize_anchor, serialize_anchor as serialize_text_anchor},
    Buffer, BufferSnapshot,
};
use rpc::{proto, AnyProtoClient, TypedEnvelope};
use settings::Settings;
use settings::WorktreeId;
use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
    num::NonZeroU32,
    ops::Range,
    path::Path,
    sync::Arc,
};
use sum_tree::TreeMap;
use text::{Anchor, Point};
use util::{maybe, ResultExt as _};

#[derive(Clone)]
struct RemoteBreakpointStore {
    upstream_client: Option<AnyProtoClient>,
    upstream_project_id: u64,
}

pub struct BreakpointStore {
    // TODO: This is.. less than ideal, as it's O(n) and does not return entries in order. We'll have to change TreeMap to support passing in the context for comparisons
    breakpoints: BTreeMap<Arc<Path>, Vec<(text::Anchor, Breakpoint)>>,
    downstream_client: Option<(AnyProtoClient, u64)>,
    active_stack_frames: HashMap<u64, (Arc<Path>, Point)>,
    // E.g ssh
    upstream_client: Option<RemoteBreakpointStore>,
}

impl BreakpointStore {
    pub(crate) fn init(client: &AnyProtoClient) {}
    pub fn local() -> Self {
        BreakpointStore {
            breakpoints: BTreeMap::new(),
            upstream_client: None,
            downstream_client: None,
            active_stack_frames: Default::default(),
        }
    }

    pub(crate) fn remote(
        upstream_project_id: u64,
        upstream_client: AnyProtoClient,
        cx: &mut Context<Self>,
    ) -> Self {
        BreakpointStore {
            breakpoints: BTreeMap::new(),
            upstream_client: Some(RemoteBreakpointStore {
                upstream_client: Some(upstream_client),
                upstream_project_id,
            }),
            downstream_client: None,
            active_stack_frames: Default::default(),
        }
    }

    pub(crate) fn shared(&mut self, project_id: u64, downstream_client: AnyProtoClient) {
        self.downstream_client = Some((downstream_client.clone(), project_id));
    }

    pub(crate) fn unshared(&mut self, cx: &mut Context<Self>) {
        self.downstream_client.take();

        cx.notify();
    }

    fn upstream_client(&self) -> Option<RemoteBreakpointStore> {
        self.upstream_client.clone()
    }

    pub fn set_active_stack_frame(
        &mut self,
        thread_id: u64,
        path: Arc<Path>,
        position: Point,
        cx: &mut Context<Self>,
    ) {
        self.active_stack_frames
            .insert(thread_id, (path.clone(), position.clone()));
    }

    pub fn toggle_breakpoint(
        &mut self,
        abs_path: Arc<Path>,
        mut breakpoint: (text::Anchor, Breakpoint),
        edit_action: BreakpointEditAction,
        cx: &mut Context<Self>,
    ) {
        let upstream_client = self.upstream_client();
        let breakpoint_set = self.breakpoints.entry(abs_path.clone()).or_default();

        match edit_action {
            BreakpointEditAction::Toggle => {
                let len_before = breakpoint_set.len();
                breakpoint_set.retain(|value| &breakpoint != value);
                if len_before == breakpoint_set.len() {
                    // We did not remove any breakpoint, hence let's toggle one.
                    breakpoint_set.push(breakpoint);
                }
            }
            BreakpointEditAction::EditLogMessage(log_message) => {
                if !log_message.is_empty() {
                    breakpoint.1.kind = BreakpointKind::Log(log_message.clone());
                    let len_before = breakpoint_set.len();
                    breakpoint_set.retain(|value| &breakpoint != value);
                    if len_before == breakpoint_set.len() {
                        // We did not remove any breakpoint, hence let's toggle one.
                        breakpoint_set.push(breakpoint);
                    }
                } else if matches!(&breakpoint.1.kind, BreakpointKind::Log(_)) {
                    breakpoint_set.retain(|value| &breakpoint != value);
                }
            }
        }

        if breakpoint_set.is_empty() {
            self.breakpoints.remove(&abs_path);
        }

        cx.notify();
    }

    pub fn on_file_rename(
        &mut self,
        old_path: Arc<Path>,
        new_path: Arc<Path>,
        cx: &mut Context<Self>,
    ) {
        if let Some(breakpoints) = self.breakpoints.remove(&old_path) {
            self.breakpoints.insert(new_path.clone(), breakpoints);

            cx.notify();
        }
    }

    pub fn breakpoints<'a>(
        &'a self,
        path: &'a Path,
        range: Option<Range<text::Anchor>>,
        buffer_snapshot: BufferSnapshot,
    ) -> impl Iterator<Item = &'a (text::Anchor, Breakpoint)> + 'a {
        self.breakpoints
            .get(path)
            .into_iter()
            .flat_map(move |breakpoints| {
                breakpoints.into_iter().filter({
                    let range = range.clone();
                    let buffer_snapshot = buffer_snapshot.clone();
                    move |(position, _)| {
                        if let Some(range) = &range {
                            position.cmp(&range.start, &buffer_snapshot).is_ge()
                                && position.cmp(&range.end, &buffer_snapshot).is_le()
                        } else {
                            false
                        }
                    }
                })
            })
    }

    pub(crate) fn all_breakpoints(
        &self,
        cx: &App,
    ) -> HashMap<Arc<Path>, Vec<SerializedBreakpoint>> {
        let all_breakpoints: HashMap<Arc<Path>, Vec<SerializedBreakpoint>> = Default::default();
        let as_abs_path = true;
        /*

        for (project_path, breakpoints) in &self.breakpoints {
            let buffer = maybe!({
                let buffer_store = self.buffer_store.read(cx);
                let buffer_id = buffer_store.buffer_id_for_project_path(project_path)?;
                let buffer = buffer_store.get(*buffer_id)?;
                Some(buffer.read(cx))
            });

            let Some(path) = maybe!({
                if as_abs_path {
                    let worktree = self
                        .worktree_store
                        .read(cx)
                        .worktree_for_id(project_path.worktree_id, cx)?;
                    Some(Arc::from(
                        worktree
                            .read(cx)
                            .absolutize(&project_path.path)
                            .ok()?
                            .as_path(),
                    ))
                } else {
                    Some(project_path.path.clone())
                }
            }) else {
                continue;
            };

            all_breakpoints.entry(path).or_default().extend(
                breakpoints
                    .into_iter()
                    .map(|bp| bp.to_serialized(buffer, project_path.clone().path)),
            );
        }*/

        all_breakpoints
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn breakpoints(&self) -> &BTreeMap<ProjectPath, HashSet<Breakpoint>> {
        &self.breakpoints
    }
}

type LogMessage = Arc<str>;

#[derive(Clone, Debug)]
pub enum BreakpointEditAction {
    Toggle,
    EditLogMessage(LogMessage),
}

#[derive(Clone, Debug)]
pub enum BreakpointKind {
    Standard,
    Log(LogMessage),
}

impl BreakpointKind {
    pub fn to_int(&self) -> i32 {
        match self {
            BreakpointKind::Standard => 0,
            BreakpointKind::Log(_) => 1,
        }
    }

    pub fn log_message(&self) -> Option<LogMessage> {
        match self {
            BreakpointKind::Standard => None,
            BreakpointKind::Log(message) => Some(message.clone()),
        }
    }
}

impl PartialEq for BreakpointKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for BreakpointKind {}

impl Hash for BreakpointKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Breakpoint {
    pub kind: BreakpointKind,
}

impl Breakpoint {
    fn to_proto(&self, path: &Path, position: &text::Anchor) -> Option<client::proto::Breakpoint> {
        Some(client::proto::Breakpoint {
            position: Some(serialize_text_anchor(position)),

            kind: match self.kind {
                BreakpointKind::Standard => proto::BreakpointKind::Standard.into(),
                BreakpointKind::Log(_) => proto::BreakpointKind::Log.into(),
            },
            message: if let BreakpointKind::Log(message) = &self.kind {
                Some(message.to_string())
            } else {
                None
            },
        })
    }

    fn from_proto(breakpoint: client::proto::Breakpoint) -> Option<Self> {
        None
        // Some(Self {
        //     position: deserialize_anchor(breakpoint.position?)?,
        //     kind: match proto::BreakpointKind::from_i32(breakpoint.kind) {
        //         Some(proto::BreakpointKind::Log) => {
        //             BreakpointKind::Log(breakpoint.message.clone().unwrap_or_default().into())
        //         }
        //         None | Some(proto::BreakpointKind::Standard) => BreakpointKind::Standard,
        //     },
        // })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SerializedBreakpoint {
    pub position: u32,
    pub path: Arc<Path>,
    pub kind: BreakpointKind,
}
