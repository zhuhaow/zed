use client::{Client, UserStore};
use clock::RealSystemClock;
use gpui::Context;
use http_client::{BlockedHttpClient, HttpClientWithUrl};
use language::{language_settings::AllLanguageSettings, LanguageRegistry, ParseStatus};
use project::project_settings::ProjectSettings;
use project::{Project, ProjectPath, WorktreeSettings};
use settings::Settings;
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    gpui::App::new().run(|cx| {
        if std::env::var("RUST_LOG").is_ok() {
            env_logger::init();
        }

        settings::init(cx);
        ProjectSettings::register(cx);
        WorktreeSettings::register(cx);
        AllLanguageSettings::register(cx);

        let http_client = Arc::new(BlockedHttpClient);
        let clock = Arc::new(RealSystemClock);
        let client = Client::new(
            clock,
            Arc::new(HttpClientWithUrl::new(http_client.clone(), "", None)),
            cx,
        );
        let user_store = cx.new_model(|app| UserStore::new(client.clone(), app));
        let fs = Arc::new(fs::RealFs::new(
            Arc::new(git::GitHostingProviderRegistry::new()),
            None,
        ));
        let (_tx, rx) = async_watch::channel(None);
        let node_runtime = node_runtime::NodeRuntime::new(http_client.clone(), rx);
        let language_registry = Arc::new(LanguageRegistry::new(cx.background_executor().clone()));

        languages::init(language_registry.clone(), node_runtime.clone(), cx);

        let project = Project::local(
            client,
            node_runtime,
            user_store,
            language_registry,
            fs,
            None,
            cx,
        );
        let worktree = project.update(cx, |project, cx| {
            let abs_path = std::fs::canonicalize("../zed").unwrap();
            project.create_worktree(abs_path, true, cx)
        });

        cx.spawn(|mut cx| async move {
            let worktree = worktree.await?;

            let (worktree_id, scan_complete) = worktree.update(&mut cx, |worktree, _cx| {
                (worktree.id(), worktree.as_local().unwrap().scan_complete())
            })?;

            scan_complete.await;
            println!("Worktree scan complete");

            // Open model_context.rs and get its outline
            let buffer = project
                .update(&mut cx, |project, cx| {
                    println!("Opening buffer");
                    project.open_buffer(
                        ProjectPath {
                            worktree_id,
                            path: Arc::from(PathBuf::from("crates/gpui/src/app/entity_map.rs")),
                        },
                        cx,
                    )
                })?
                .await?;

            println!("Opened buffer");

            let mut parse_status = buffer.read_with(&cx, |buffer, _cx| buffer.parse_status())?;
            loop {
                if parse_status.recv().await? == ParseStatus::Idle {
                    break;
                }
            }

            println!("Parsing complete");

            buffer.update(&mut cx, |buffer, _cx| {
                for layer in buffer.snapshot().syntax_layers() {
                    println!("Layer: {:?}", layer);
                }

                let outline = buffer.snapshot().outline(None).unwrap();

                for item in outline.items {
                    println!("{}", item.text);
                }
            })?;

            anyhow::Ok(())
        })
        .detach();
    });
}
