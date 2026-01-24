use crate::editor::open_editor_for_task;
use crate::orchestrator::{LaunchRequest, Orchestrator};
use crate::provider::Provider;
use crate::Result;

pub struct LaunchOptions {
    pub task: Option<String>,
    pub editor: Option<String>,
    pub branch: Option<String>,
    pub base: Option<String>,
    pub provider: Provider,
    pub code: bool,
    pub dangerously_allow_all: bool,
    pub enable_edits: bool,
    pub provider_args: Vec<String>,
}

pub async fn run(options: LaunchOptions) -> Result<()> {
    let LaunchOptions {
        task,
        editor,
        branch,
        base,
        provider,
        code,
        dangerously_allow_all,
        enable_edits,
        provider_args,
    } = options;

    // Resolve the task: either from --task, --editor, or error
    let task = match (task, editor) {
        (Some(t), None) => t,
        (_, Some(cmd)) => {
            let editor_cmd = if cmd.is_empty() { None } else { Some(cmd) };
            open_editor_for_task(editor_cmd)?
        }
        (None, None) => {
            eprintln!("Error: Either --task or --editor must be provided");
            eprintln!("  Use --task \"description\" for inline task");
            eprintln!("  Use --editor [cmd] to compose in your editor");
            std::process::exit(1);
        }
    };

    let mut orchestrator = Orchestrator::new()?;

    let mut provider_args = provider_args;
    if dangerously_allow_all {
        match provider {
            Provider::Claude | Provider::Amp => {
                provider_args.insert(0, "--dangerously-allow-all".to_string());
            }
            _ => {}
        }
    }

    if enable_edits {
        match provider {
            Provider::Claude => {
                provider_args.insert(0, "--enable-edits".to_string());
            }
            _ => {}
        }
    }

    let request = LaunchRequest {
        task: task.clone(),
        branch,
        base,
        provider,
        provider_args,
    };

    let id = orchestrator.launch(request).await?;

    if code {
        orchestrator.open_vscode(&id.to_string())?;
    }

    println!("Launched agent {id} on branch wta/{id}");
    println!("Provider: {provider}");
    println!("Task: {task}");
    println!();
    println!("Use 'wta attach {id}' to watch the agent");
    println!("Use 'wta status {id}' to check progress");

    if code {
        orchestrator.open_vscode(&id.0)?;
    }

    Ok(())
}
