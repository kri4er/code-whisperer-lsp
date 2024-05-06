use std::collections::HashMap;

use aiutils::{AWSCodeWhispererCliProvider, RecommendationProvider};
use codespan_lsp::position_to_byte_index;
use codespan::{FileId, Files};
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::aiutils::{FileContext, ProgrammingLanguage, RecommendationContext};

mod aiutils;
pub mod codefileutil;
mod codewshiperer;

#[derive(Debug)]
struct State {
    pub sources: HashMap<Url, FileId>,
    pub files: Files<String>,
    pub rec_provider: AWSCodeWhispererCliProvider,
}

#[derive(Debug)]
struct Backend {
    editor_client: Client,
    state: Mutex<State>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.editor_client
            .log_message(
                MessageType::WARNING,
                "LOGME: AWS Code whisperer server initialized!",
            )
            .await;

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: None,
                // definition: Some(GotoCapability::default()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut state = self.state.lock().await;
        let document = &params.text_document;
        if state.sources.get(&params.text_document.uri).is_none() {
            let id = state
                .files
                .add(document.uri.to_string(), document.text.clone());
            state.sources.insert(document.uri.clone(), id);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::info!(
            "Got a textDocument/didSave notification, here are params: {:?}",
            params
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::info!(
            "Got a textDocument/didChange notification, here are params: {:?}",
            params
        );
        let mut state = self.state.lock().await;
        reload_source(&mut state, &params.text_document, params.content_changes)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        tracing::info!(
            "Got a textDocument/completion notification, with params: {:?}",
            params
        );
        let state = self.state.lock().await;
        let uri = &params.text_document_position.text_document.uri;
        let position = &params.text_document_position.position;
        if let Some(document) = state.sources.get(uri) {
            let source = state.files.source(*document);
            let source:&String = source;
            let span = state.files.line_span(*document, position.line).unwrap();
            tracing::info!(
                "textDocument/completion source: {:?} and line span: {:?}, str len: {:?}, \nleft: {:?}\n{:?}, right:",
                source,
                span,
                source.len(),
                source.get(..span.end().0 as usize),
                source.get(span.end().0 as usize..),
                );

            //let line_str = source.get(&span.start() as u32).unwrap();

            let context = RecommendationContext {
                file_context: FileContext {
                    filename: "main.rs".into(),
                    programming_language: ProgrammingLanguage {
                        language_name: "rust".into(),
                    },
                    left_file_content: source.get(..span.end().0 as usize).unwrap().to_string(),
                    right_file_content: source.get(span.end().0 as usize..).unwrap().to_string(),
                },
                max_results: 3,
            };

            tracing::info!(
                "textDocument/completion calling get recomendations with context: {:?}",
                context
                );

            let res = state.rec_provider.recomendations(context);
            tracing::info!("Got next reccomendations: {:?}", res);

            let reccomendations: Vec<CompletionItem> = res
                .into_iter()
                .map(|rec| CompletionItem::new_simple(rec.content.clone(), rec.content))
                .collect();
            return Ok(Some(CompletionResponse::from(
                        reccomendations
                        )))
        } else {
            return Ok(Some(CompletionResponse::from(
                        vec![
                        CompletionItem::new_simple("Yurii, go back to BACKFILL".into(), "GO BACK TO BACKFILL DUDE".into())
                        ]
                        )))
        }
    }

    async fn initialized(&self, _: InitializedParams) {
        println!("LOGME: printAWS Code Whisperer is initialized now.");
        self.editor_client
            .log_message(
                MessageType::WARNING,
                "LOGME: AWS Code whisperer server initialized!",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down the server.");
        Ok(())
    }
}

fn reload_source(
    state: &mut State,
    document: &VersionedTextDocumentIdentifier,
    changes: Vec<TextDocumentContentChangeEvent>,
) {
    if let Some(id) = state.sources.get(&document.uri) {
        let mut source = state.files.source(*id).to_owned();
        for change in changes {
            if let (None, None) = (change.range, change.range_length) {
                source = change.text;
            } else if let Some(range) = change.range {
                panic!("attempted to reload source using range update, it is not supported.");
            }
        }
        state.files.update(*id, source);
    } else {
        panic!("attempted to reload source that does not exist");
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let file_appender = tracing_appender::rolling::hourly("/tmp/debug_code_whisperer", "test1.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
    let cli_rec_provider = AWSCodeWhispererCliProvider {};

    let (service, socket) = LspService::new(|editor_client| Backend {
        editor_client,
        state: Mutex::new(State {
            rec_provider: cli_rec_provider.into(),
            sources: HashMap::new(),
            files: Files::new(),
        }),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
