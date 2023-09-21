use std::{path::PathBuf, rc::Rc, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::Context;
use tracing::{debug, error};

use self::{
    bridge::procedure_call::{emit_result, next_procedure, proc_bridge_ready, ProcedureCallInner},
    r#macro::GenericMainWorkerGenerator,
};
use crate::{
    error::Error,
    event_broadcaster::EventBroadcaster,
    events::CausedBy,
    macro_executor::{self, MacroExecutor, MacroPID, SpawnResult, WorkerOptionGenerator},
    traits::{
        t_configurable::{
            manifest::{SetupManifest, SetupValue},
            TConfigurable,
        },
        t_player::TPlayerManagement,
        t_server::TServer,
        InstanceInfo, TInstance,
    },
    types::DotLodestoneConfig,
};
use std::io::Write;

mod bridge;
pub mod configurable;
mod r#macro;
pub mod player;
pub mod resource;
pub mod server;

#[derive(Clone)]
pub struct GenericInstance {
    dot_lodestone_config: DotLodestoneConfig,
    event_broadcaster: EventBroadcaster,
    procedure_bridge: bridge::procedure_call::ProcedureBridge,
    core_macro_executor: MacroExecutor,
    path: PathBuf,
    core_macro_pid: MacroPID,
    drop_guard: Arc<GenericDropGuard>,
}

/// RAII guard for dropping a generic instance
///
/// Will abort the core macro process if the instance is dropped
struct GenericDropGuard {
    core_macro_pid: MacroPID,
    macro_executor: MacroExecutor,
}

impl Drop for GenericDropGuard {
    fn drop(&mut self) {
        debug!("Dropping generic instance, aborting core macro process");
        let _ = self.macro_executor.abort_macro(self.core_macro_pid);
    }
}

struct InitWorkerGenerator {
    pub bridge: bridge::procedure_call::ProcedureBridge,
}

impl WorkerOptionGenerator for InitWorkerGenerator {
    fn generate(&self) -> deno_runtime::worker::WorkerOptions {
        let ext = deno_core::Extension::builder("generic_deno_extension_builder")
            .ops(vec![
                next_procedure::decl(),
                emit_result::decl(),
                proc_bridge_ready::decl(),
            ])
            .state({
                let brige = self.bridge.clone();
                move |state| {
                    state.put(brige);
                }
            })
            .build();
        deno_runtime::worker::WorkerOptions {
            extensions: vec![ext],
            module_loader: Rc::new(macro_executor::TypescriptModuleLoader::default()),
            ..Default::default()
        }
    }
}

impl GenericInstance {
    pub async fn new(
        link_to_source: String,
        path: PathBuf,
        dot_lodestone_config: DotLodestoneConfig,
        setup_value: SetupValue,
        event_broadcaster: EventBroadcaster,
        core_macro_executor: MacroExecutor,
    ) -> Result<Self, Error> {
        tokio::fs::create_dir_all(&path).await.context(format!(
            "Failed to create directory for instance at {}",
            &path.display()
        ))?;
        let path_to_config = path.join(".lodestone_config");
        let run_ts_content =
            include_str!("js/main/bootstrap.ts").replace("REPLACE_ME_WITH_URL", &link_to_source);

        let path_to_bootstrap = path.join("run.ts");
        tokio::fs::write(&path_to_bootstrap, run_ts_content)
            .await
            .context(format!(
                "Failed to write bootstrap to {}",
                &path_to_bootstrap.display()
            ))?;
        tokio::fs::write(
            &path_to_config,
            serde_json::to_string_pretty(&dot_lodestone_config).context(
                "Failed to serialize config to string. This is a bug, please report it.",
            )?,
        )
        .await
        .context(format!(
            "Failed to write config to {}",
            &path_to_config.display()
        ))?;

        let procedure_bridge = bridge::procedure_call::ProcedureBridge::new();

        let SpawnResult {
            macro_pid: core_macro_pid,
            detach_future,
            ..
        } = core_macro_executor
            .spawn(
                path_to_bootstrap,
                Vec::new(),
                CausedBy::System,
                Box::new(GenericMainWorkerGenerator::new(procedure_bridge.clone())),
                None,
                Some(dot_lodestone_config.uuid().clone()),
            )
            .await?;
        detach_future.await;
        procedure_bridge
            .call(ProcedureCallInner::SetupInstance {
                dot_lodestone_config: dot_lodestone_config.clone(),
                setup_value,
                path: path.clone(),
            })
            .await?;
        Ok(GenericInstance {
            dot_lodestone_config,
            procedure_bridge,
            event_broadcaster,
            core_macro_executor: core_macro_executor.clone(),
            path,
            core_macro_pid,
            drop_guard: Arc::new(GenericDropGuard {
                core_macro_pid,
                macro_executor: core_macro_executor,
            }),
        })
    }

    pub async fn restore(
        path_to_instance: PathBuf,
        dot_lodestone_config: DotLodestoneConfig,
        event_broadcaster: EventBroadcaster,
        core_macro_executor: MacroExecutor,
    ) -> Result<Self, Error> {
        let procedure_bridge = bridge::procedure_call::ProcedureBridge::new();
        let SpawnResult {
            macro_pid: core_macro_pid,
            detach_future,
            exit_future,
        } = core_macro_executor
            .spawn(
                path_to_instance.join("run.ts"),
                Vec::new(),
                CausedBy::System,
                Box::new(GenericMainWorkerGenerator::new(procedure_bridge.clone())),
                None,
                Some(dot_lodestone_config.uuid().clone()),
            )
            .await?;

        tokio::select! {
            _ = detach_future => {
            }
            res = exit_future => {
                debug!("Core macro exited with {:?}", res);
                res?;
            }
        }

        procedure_bridge
            .call(ProcedureCallInner::RestoreInstance {
                dot_lodestone_config: dot_lodestone_config.clone(),
                path: path_to_instance.clone(),
            })
            .await?;
        Ok(GenericInstance {
            dot_lodestone_config,
            procedure_bridge,
            event_broadcaster,
            core_macro_executor: core_macro_executor.clone(),
            path: path_to_instance,
            core_macro_pid,
            drop_guard: Arc::new(GenericDropGuard {
                core_macro_pid,
                macro_executor: core_macro_executor,
            }),
        })
    }

    pub async fn setup_manifest(
        link_to_source: &str,
        macro_executor: MacroExecutor,
    ) -> Result<SetupManifest, Error> {
        // create a tempfile
        let temp_dir = tempfile::TempDir::new().context("Failed to create temp dir")?;
        let temp_file_path = temp_dir.path().join("temp.ts");
        let mut temp_file =
            std::fs::File::create(&temp_file_path).context("Failed to create temp file")?;
        let run_ts_content =
            include_str!("js/main/bootstrap.ts").replace("REPLACE_ME_WITH_URL", link_to_source);
        println!("{}", run_ts_content);
        writeln!(temp_file, "{}", run_ts_content).context("Failed to write to temp file")?;
        let procedure_bridge = bridge::procedure_call::ProcedureBridge::new();
        let SpawnResult {
            macro_pid,
            detach_future,
            exit_future,
        } = macro_executor
            .spawn(
                temp_file_path,
                Vec::new(),
                CausedBy::System,
                Box::new(InitWorkerGenerator {
                    bridge: procedure_bridge.clone(),
                }),
                None,
                None,
            )
            .await?;

        tokio::select! {
            _ = detach_future => {
            }
            res = exit_future => {
                res?;
            }
        }

        let ret = procedure_bridge
            .call(ProcedureCallInner::GetSetupManifest)
            .await?
            .try_into();
        let _ = macro_executor.abort_macro(macro_pid);
        ret
    }

    /// Will notify the typescript side that the instance is being destructed
    pub async fn destruct(self) {
        let _ = self
            .procedure_bridge
            .call(ProcedureCallInner::DestructInstance)
            .await
            .map_err(|e| {
                error!("Generic instance destructor raised an error: {}", e);
            });
    }
}

#[async_trait]
impl TInstance for GenericInstance {
    async fn get_instance_info(&self) -> InstanceInfo {
        InstanceInfo {
            uuid: self.uuid().await,
            name: self.name().await,
            game_type: self.game_type().await,
            description: self.description().await,
            version: self.version().await,
            port: self.port().await,
            creation_time: self.creation_time().await,
            path: self.path().await.display().to_string(),
            auto_start: self.auto_start().await,
            restart_on_crash: self.restart_on_crash().await,
            state: self.state().await,
            player_count: self.get_player_count().await.ok(),
            max_player_count: self.get_max_player_count().await.ok(),
            player_list: self.get_player_list().await.ok(),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::path::PathBuf;

//     use crate::{
//         event_broadcaster::EventBroadcaster,
//         events::CausedBy,
//         implementations::generic::GenericInstance,
//         macro_executor::MacroExecutor,
//         types::{DotLodestoneConfig, InstanceUuid},
//     };

//     use crate::traits::t_configurable::GameType;

//     #[tokio::test]
//     async fn test_create_generic_instance() {
//         let _ = tracing_subscriber::fmt::try_init();
//         let (event_tx, mut rx) = EventBroadcaster::new(100);
//         let core_macro_executor = MacroExecutor::new(event_tx.clone());
//         let manifest = GenericInstance::setup_manifest(
//             "https://raw.githubusercontent.com/CheatCod/generic_instance_test/main/",
//             core_macro_executor,
//         )
//         .await
//         .unwrap();

//         println!("{:?}", manifest);

//         // tokio::time::sleep(std::time::Duration::from_secs(100)).await;
//     }
// }
