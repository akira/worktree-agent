use crate::Result;

const SHELL_FUNCTION: &str = r#"# WTA shell function - enables directory switching
# Add this to your .bashrc or .zshrc

wta() {
    local directive_file exit_code=0
    directive_file="$(mktemp)"

    WTA_DIRECTIVE_FILE="$directive_file" command wta "$@" || exit_code=$?

    if [[ -s "$directive_file" ]]; then
        source "$directive_file"
        if [[ $exit_code -eq 0 ]]; then
            exit_code=$?
        fi
    fi

    rm -f "$directive_file"
    return "$exit_code"
}
"#;

pub async fn run() -> Result<()> {
    println!("{SHELL_FUNCTION}");
    Ok(())
}
