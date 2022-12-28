use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use log::{debug, error, info};
use tokio::{
    runtime::Builder,
    sync::{broadcast, mpsc, oneshot, Mutex},
    task::{JoinHandle, LocalSet},
};

use crate::{
    events::{MacroEvent, MacroEventInner},
    traits::{Error, ErrorInner},
    types::InstanceUuid,
};

use std::pin::Pin;

use anyhow::bail;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::anyhow;
use deno_core::anyhow::anyhow;
use deno_core::resolve_import;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use futures::FutureExt;
pub struct TypescriptModuleLoader;

pub fn resolve_macro_invocation(
    path_to_macro: &Path,
    macro_name: &str,
    is_in_game: bool,
) -> Option<PathBuf> {
    let path_to_macro = if is_in_game {
        path_to_macro.join("in_game")
    } else {
        path_to_macro.to_owned()
    };

    let ts_macro = path_to_macro.join(macro_name).with_extension("ts");
    let js_macro = path_to_macro.join(macro_name).with_extension("js");

    let macro_folder = path_to_macro.join(macro_name);

    if ts_macro.is_file() {
        return Some(ts_macro);
    } else if js_macro.is_file() {
        return Some(js_macro);
    } else if macro_folder.is_dir() {
        // check if index.ts exists
        let index_ts = macro_folder.join("index.ts");
        let index_js = macro_folder.join("index.js");
        if index_ts.exists() {
            return Some(index_ts);
        } else if index_js.exists() {
            return Some(index_js);
        }
    } else if !is_in_game {
        return resolve_macro_invocation(&path_to_macro.join("in_game"), macro_name, true);
    };
    None
}

impl ModuleLoader for TypescriptModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| anyhow!("Only file: URLs are supported."))?;

            let path = if path.extension().is_none() && path.with_extension("ts").exists() {
                path.with_extension("ts")
            } else if path.with_extension("js").exists() {
                path.with_extension("js")
            } else {
                path
            };

            let path = if path.is_dir() {
                // check if index.ts exists
                let index_ts = path.join("index.ts");
                if index_ts.exists() {
                    index_ts
                } else {
                    path.join("index.js")
                }
            } else {
                path
            };

            let media_type = MediaType::from(&path);
            let (module_type, should_transpile) = match MediaType::from(&path) {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => bail!("Unknown extension {:?}", path.extension()),
            };

            let code = std::fs::read_to_string(&path)?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.to_string(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                parsed.transpile(&Default::default())?.text
            } else {
                code
            };
            let module = ModuleSource {
                code: code.into_bytes().into_boxed_slice(),
                module_type,
                module_url_specified: module_specifier.to_string(),
                module_url_found: module_specifier.to_string(),
            };
            Ok(module)
        }
        .boxed_local()
    }
}

pub struct ExecutionInstruction {
    pub runtime: Box<
        dyn Fn(
                String,
                String,
                Vec<String>,
                bool,
            ) -> Result<(deno_runtime::worker::MainWorker, PathBuf), Error>
            + Send,
    >,
    pub name: String,
    pub executor: Option<String>,
    pub args: Vec<String>,
    pub is_in_game: bool,
    pub instance_uuid: InstanceUuid,
}

pub enum Instruction {
    Spawn(ExecutionInstruction),
    Abort(usize),
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Spawn(exec_instruction) => f
                .debug_struct("Instruction::Spawn")
                .field("name", &exec_instruction.name)
                .field("args", &exec_instruction.args)
                .field("executor", &exec_instruction.executor)
                .finish(),
            Instruction::Abort(uuid) => {
                write!(f, "Abort {{ uuid: {} }}", uuid)
            }
        }
    }
}

#[derive(Clone)]
pub struct MacroExecutor {
    macro_process_table: Arc<Mutex<HashMap<usize, (JoinHandle<()>, deno_core::v8::IsolateHandle)>>>,
    sender: mpsc::UnboundedSender<(Instruction, usize)>,
    event_broadcaster: broadcast::Sender<MacroEvent>,
    next_process_id: Arc<AtomicUsize>,
}

impl MacroExecutor {
    pub fn new() -> MacroExecutor {
        let (tx, mut rx): (
            mpsc::UnboundedSender<(Instruction, usize)>,
            mpsc::UnboundedReceiver<(Instruction, usize)>,
        ) = mpsc::unbounded_channel();
        let (event_broadcaster, _) = broadcast::channel(16);
        let process_table = Arc::new(Mutex::new(HashMap::new()));
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let process_id = Arc::new(AtomicUsize::new(0));
        std::thread::spawn({
            let process_table = process_table.clone();
            let process_id = process_id.clone();
            let event_broadcaster = event_broadcaster.clone();
            move || {
                let local = LocalSet::new();
                local.spawn_local(async move {
                    while let Some((new_task, pid)) = rx.recv().await {
                        match new_task {
                            Instruction::Spawn(exec_instruction) => {
                                let ExecutionInstruction {
                                    runtime,
                                    name,
                                    args,
                                    executor,
                                    is_in_game,
                                    instance_uuid,
                                } = exec_instruction;
                                let executor = executor.unwrap_or_default();
                                let (mut runtime, path_to_main_module) =
                                    match runtime(name, executor, args, is_in_game) {
                                        Ok((runtime, path_to_main_module)) => {
                                            (runtime, path_to_main_module)
                                        }
                                        Err(e) => {
                                            error!("Error creating runtime: {}", e);
                                            continue;
                                        }
                                    };
                                let isolate_handle =
                                    runtime.js_runtime.v8_isolate().thread_safe_handle();
                                let handle = tokio::task::spawn_local({
                                    let event_broadcaster = event_broadcaster.clone();
                                    let process_id = process_id.clone();
                                    async move {
                                        let main_module = match deno_core::resolve_path(
                                            &path_to_main_module.to_string_lossy(),
                                        ) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!("Error resolving main module: {}", e);
                                                return;
                                            }
                                        };

                                        let _ = runtime
                                            .execute_main_module(&main_module)
                                            .await
                                            .map_err(|e| {
                                                error!("Error executing main module: {}", e);
                                                e
                                            });

                                        let _ = runtime.run_event_loop(false).await.map_err(|e| {
                                            error!("Error while running event loop: {}", e);
                                        });

                                        let _ = event_broadcaster.send(MacroEvent {
                                            macro_pid: process_id.load(Ordering::SeqCst),
                                            macro_event_inner: MacroEventInner::MacroStopped,
                                            instance_uuid,
                                        });
                                    }
                                });
                                process_table
                                    .lock()
                                    .await
                                    .insert(pid, (handle, isolate_handle));
                            }
                            Instruction::Abort(pid) => {
                                if let Some((_, isolate_handle)) =
                                    process_table.lock().await.get(&pid)
                                {
                                    isolate_handle.terminate_execution();
                                }
                            }
                        }
                    }
                    // If the while loop returns, then all the LocalSpawner
                    // objects have been dropped.
                });

                // This will return once all senders are dropped and all
                // spawned tasks have returned.
                rt.block_on(local);
                debug!("MacroExecutor thread exited");
            }
        });
        MacroExecutor {
            macro_process_table: process_table,
            sender: tx,
            event_broadcaster,
            next_process_id: process_id,
        }
    }

    pub fn event_receiver(&self) -> broadcast::Receiver<MacroEvent> {
        self.event_broadcaster.subscribe()
    }

    pub fn spawn(&self, exec_instruction: ExecutionInstruction) -> usize {
        let pid = self.next_process_id.fetch_add(1, Ordering::SeqCst);
        self.sender
            .send((Instruction::Spawn(exec_instruction), pid))
            .expect("Thread with LocalSet has shut down.");
        info!("Spawned macro with pid {}", pid);
        pid
    }

    /// abort a macro execution
    pub async fn abort_macro(&self, pid: &usize) -> Result<(), Error> {
        self.macro_process_table
            .lock()
            .await
            .get(pid)
            .ok_or_else(|| Error {
                inner: ErrorInner::MacroNotFound,
                detail: "Macro not found".to_owned(),
            })?
            .1
            .terminate_execution();
        Ok(())
    }
    /// wait for a macro to finish
    ///
    /// if timeout is None, wait forever
    ///
    /// if timeout is Some, wait for the specified amount of time
    ///
    /// returns true if the macro finished, false if the timeout was reached
    pub async fn wait_with_timeout(&self, macro_pid: usize, timeout: Option<f64>) -> bool {
        let mut rx = self.event_broadcaster.subscribe();
        tokio::select! {
            _ = async {
                if let Some(timeout) = timeout {
                    tokio::time::sleep(Duration::from_secs_f64(timeout)).await;
                } else {
                    // create a future that never resolves
                    let (_tx, rx) = oneshot::channel::<()>();
                    let _ = rx.await;

                }
            } => {
                false
            }
            _ = {
                async {
                    loop {
                    let event = rx.recv().await.unwrap();
                    if let MacroEventInner::MacroStopped = event.macro_event_inner {
                        if event.macro_pid == macro_pid {
                            break;
                        }
                    }
                }
            }} => {
                true
            }
        }
    }

    pub async fn get_macro_status(&self, pid: usize) -> Result<bool, Error> {
        let table = self.macro_process_table.lock().await;
        let handle = table.get(&pid).ok_or_else(|| Error {
            inner: ErrorInner::MacroNotFound,
            detail: "Macro not found".to_owned(),
        })?;
        Ok(handle.0.is_finished())
    }
}

impl Default for MacroExecutor {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    use std::{path::PathBuf, rc::Rc};

    use crate::{types::InstanceUuid, Error};

    use super::{resolve_macro_invocation, TypescriptModuleLoader};

    #[tokio::test]
    async fn test_macro_executor() {
        // construct a macro executor
        let executor = super::MacroExecutor::new();

        // create a temp directory
        let path_to_macros = tempdir::TempDir::new("macro_executor_test")
            .unwrap()
            .into_path();
        // create test js file

        let runtime = Box::new({
            let path_to_macros = path_to_macros.clone();
            move |macro_name: String,
                  _executor: String,
                  args: Vec<String>,
                  is_in_game: bool|
                  -> Result<(deno_runtime::worker::MainWorker, PathBuf), Error> {
                let path_to_main_module =
                    resolve_macro_invocation(&path_to_macros, &macro_name, is_in_game)
                        .expect("Failed to resolve macro invocation");

                let bootstrap_options = deno_runtime::BootstrapOptions {
                    args,
                    ..Default::default()
                };

                let mut worker_options = deno_runtime::worker::WorkerOptions {
                    bootstrap: bootstrap_options,
                    ..Default::default()
                };

                worker_options.module_loader = Rc::new(TypescriptModuleLoader);
                let main_module = deno_core::resolve_path(&path_to_main_module.to_string_lossy())
                    .expect("Failed to resolve path");
                // todo(CheatCod3) : limit the permissions
                let permissions = deno_runtime::permissions::Permissions::allow_all();
                let worker = deno_runtime::worker::MainWorker::bootstrap_from_options(
                    main_module,
                    permissions,
                    worker_options,
                );

                Ok((worker, path_to_main_module))
            }
        });

        let path_to_basic_js = path_to_macros.join("basic.js");

        std::fs::write(path_to_basic_js, "console.log('hello world')").unwrap();

        let instruction = super::ExecutionInstruction {
            runtime: runtime.clone(),
            name: "basic".to_owned(),
            args: vec![],
            executor: None,
            is_in_game: false,
            instance_uuid: InstanceUuid::default(),
        };
        executor.spawn(instruction);


        let path_to_loop_js = path_to_macros.join("loop.js");

        std::fs::write(
            path_to_loop_js,
            "
            console.log('starting loop');
            for (let i = 0; i < 1000; i++) {
                // await new Promise(r => setTimeout(r, 0));
                console.log(i);
            }",
        )
        .unwrap();

        for _ in 0..1000 {
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(0.1)).await;
            let instruction = super::ExecutionInstruction {
                runtime: runtime.clone(),
                name: "loop".to_owned(),
                args: vec![],
                executor: None,
                is_in_game: false,
                instance_uuid: InstanceUuid::default(),
            };
            executor.spawn(instruction);
        }

        // let pid = executor.spawn(instruction);

        // tokio::time::sleep(tokio::time::Duration::from_secs_f64(0.01)).await;

        // executor.abort_macro(&pid).await.unwrap();

        // tokio::time::sleep(tokio::time::Duration::from_secs_f64(0.001)).await;

        // let instruction = super::ExecutionInstruction {
        //     runtime: runtime.clone(),
        //     name: "loop".to_owned(),
        //     args: vec![],
        //     executor: None,
        //     is_in_game: false,
        //     instance_uuid: InstanceUuid::default(),
        // };
        // executor.spawn(instruction);
        // println!("{}", executor.wait_with_timeout(1, Some(1.0)).await);

        tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
    }
}
