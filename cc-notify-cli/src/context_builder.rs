use crate::notify;
use crate::HookInput;

pub(crate) struct SendContext {
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub cwd: Option<String>,
    pub terminal: notify::TerminalContext,
    pub metadata: serde_json::Value,
    pub terminal_jump_command: Option<String>,
}

fn env_first(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| std::env::var(key).ok())
        .filter(|value| !value.trim().is_empty())
}

fn first_non_empty(values: Vec<Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn build_terminal_context(
    terminal_program: Option<String>,
    terminal_id: Option<String>,
    terminal_window_id: Option<String>,
    terminal_tab_id: Option<String>,
    terminal_pane_id: Option<String>,
    stdin: &HookInput,
) -> notify::TerminalContext {
    let program = first_non_empty(vec![
        terminal_program,
        stdin.terminal_program.clone(),
        env_first(&["TERM_PROGRAM", "TERMINAL_EMULATOR"]),
    ]);

    let id = first_non_empty(vec![
        terminal_id,
        stdin.terminal_id.clone(),
        env_first(&[
            "ITERM_SESSION_ID",
            "TERM_SESSION_ID",
            "WEZTERM_PANE",
            "KITTY_WINDOW_ID",
            "WT_SESSION",
            "TMUX_PANE",
        ]),
    ]);

    let window_id = first_non_empty(vec![
        terminal_window_id,
        stdin.terminal_window_id.clone(),
        env_first(&["KITTY_WINDOW_ID", "WT_SESSION"]),
    ]);

    let tab_id = first_non_empty(vec![
        terminal_tab_id,
        stdin.terminal_tab_id.clone(),
        env_first(&["TERM_SESSION_ID"]),
    ]);

    let pane_id = first_non_empty(vec![
        terminal_pane_id,
        stdin.terminal_pane_id.clone(),
        env_first(&["WEZTERM_PANE", "TMUX_PANE"]),
    ]);

    notify::TerminalContext {
        program,
        terminal_id: id,
        window_id,
        tab_id,
        pane_id,
    }
}

fn build_metadata(
    input: Option<&str>,
    session_id: Option<&str>,
    stdin: &HookInput,
    terminal: &notify::TerminalContext,
) -> serde_json::Value {
    let parsed = input
        .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
        .unwrap_or(serde_json::Value::Null);

    let mut obj = match parsed {
        serde_json::Value::Object(map) => map,
        serde_json::Value::Null => serde_json::Map::new(),
        other => {
            let mut map = serde_json::Map::new();
            map.insert("_input_metadata".to_string(), other);
            map
        }
    };

    if let Some(id) = session_id {
        obj.insert("session_id".to_string(), serde_json::json!(id));
    }
    if let Some(path) = stdin.transcript_path.as_deref() {
        obj.insert("transcript_path".to_string(), serde_json::json!(path));
    }
    if let Some(event) = stdin.hook_event_name.as_deref() {
        obj.insert("hook_event_name".to_string(), serde_json::json!(event));
    }
    if let Some(agent_id) = stdin.agent_id.as_deref() {
        obj.insert("agent_id".to_string(), serde_json::json!(agent_id));
    }

    if terminal.program.is_some()
        || terminal.terminal_id.is_some()
        || terminal.window_id.is_some()
        || terminal.tab_id.is_some()
        || terminal.pane_id.is_some()
    {
        obj.insert("terminal".to_string(), serde_json::json!(terminal));
    }

    serde_json::Value::Object(obj)
}

pub(crate) fn build_send_context(
    stdin: &HookInput,
    session_id: Option<String>,
    project: Option<String>,
    metadata: Option<&str>,
    terminal_program: Option<String>,
    terminal_id: Option<String>,
    terminal_window_id: Option<String>,
    terminal_tab_id: Option<String>,
    terminal_pane_id: Option<String>,
    terminal_jump_command: Option<String>,
) -> SendContext {
    let resolved_session_id = session_id.or(stdin.session_id.clone());
    let resolved_cwd = stdin.cwd.clone();
    let resolved_project =
        project.or_else(|| resolved_cwd.as_deref().map(notify::project_name_from_cwd));
    let terminal = build_terminal_context(
        terminal_program,
        terminal_id,
        terminal_window_id,
        terminal_tab_id,
        terminal_pane_id,
        stdin,
    );
    let resolved_terminal_jump_command = first_non_empty(vec![
        terminal_jump_command,
        stdin.terminal_jump_command.clone(),
        env_first(&["CC_NOTIFY_TERMINAL_JUMP_CMD"]),
    ]);
    let metadata = build_metadata(metadata, resolved_session_id.as_deref(), stdin, &terminal);

    SendContext {
        session_id: resolved_session_id,
        project: resolved_project,
        cwd: resolved_cwd,
        terminal,
        metadata,
        terminal_jump_command: resolved_terminal_jump_command,
    }
}
