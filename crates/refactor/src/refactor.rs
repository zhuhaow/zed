use anyhow::anyhow;
use client::{Client, UserStore};
use clock::RealSystemClock;
use gpui::Context;
use http_client::{BlockedHttpClient, HttpClientWithUrl};
use language::ToOffset;
use language::{language_settings::AllLanguageSettings, LanguageRegistry, ParseStatus};
use project::project_settings::ProjectSettings;
use project::{LspStoreEvent, Project, ProjectPath, WorktreeSettings};
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
        let (worktree, lsp_store) = project.update(cx, |project, cx| {
            let abs_path = std::fs::canonicalize("../zed").unwrap();
            let worktree = project.create_worktree(abs_path, true, cx);
            let lsp_store = project.lsp_store();

            cx.subscribe(&lsp_store, |_, _, event: &LspStoreEvent, cx| {
                dbg!(event);
            })
            .detach();

            (worktree, lsp_store)
        });

        // Keep these alive the lifetime of the app.
        std::mem::forget(lsp_store.clone());
        std::mem::forget(project.clone());

        cx.spawn(|mut cx| async move {
            {
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

                let mut parse_status =
                    buffer.read_with(&cx, |buffer, _cx| buffer.parse_status())?;
                loop {
                    if parse_status.recv().await? == ParseStatus::Idle {
                        break;
                    }
                }

                println!("Parsing complete");

                let language_server_handle = lsp_store
                    .update(&mut cx, |lsp_store, cx| {
                        lsp_store.register_buffer_with_language_servers(&buffer, cx)
                    })
                    .unwrap();
                std::mem::forget(language_server_handle);

                let outline_item = buffer
                    .update(&mut cx, |b, _cx| {
                        b.snapshot()
                            .outline(None)
                            .unwrap()
                            .items
                            .iter()
                            .find(|item| item.text == "pub fn update")
                            .cloned()
                    })?
                    .ok_or_else(|| anyhow!("No update function found"))?;

                println!("Fetching references");

                let references = project
                    .update(&mut cx, |project, cx| {
                        let mut position = outline_item.range.start.to_offset(buffer.read(cx));
                        position += outline_item.name_ranges.first().unwrap().start;
                        project.references(&buffer, position, cx)
                    })?
                    .await?;

                println!("References: {:?}", references);

                anyhow::Ok(())
            }
        })
        .detach();

        dbg!("ABOUT TO FINISH INITIALIZATION OF APP");
    });
}
