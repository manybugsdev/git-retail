use std::process::Command;

const SSH_HOST: &str = "git-retailer";
const REPOS_DIR: &str = "repos";

/// Validates a repository name to prevent shell injection.
/// Only allows alphanumeric characters, hyphens, underscores, and dots.
pub fn validate_repo_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Repository name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Repository name is too long".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(
            "Repository name may only contain alphanumeric characters, hyphens, underscores, and dots"
                .to_string(),
        );
    }
    if name.starts_with('.') {
        return Err("Repository name cannot start with a dot".to_string());
    }
    Ok(())
}

/// Returns the bare repo path on the remote host.
fn repo_path(name: &str) -> String {
    format!("{REPOS_DIR}/{name}.git")
}

/// Lists all bare repositories on the remote host.
pub fn list_repos() -> Result<Vec<String>, String> {
    let output = Command::new("ssh")
        .args([
            SSH_HOST,
            &format!("ls -1 '{REPOS_DIR}/' 2>/dev/null || true"),
        ])
        .output()
        .map_err(|e| format!("Failed to run ssh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SSH command failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<String> = stdout
        .lines()
        .filter(|line| line.ends_with(".git"))
        .map(|line| line.trim_end_matches(".git").to_string())
        .collect();

    Ok(repos)
}

/// Creates a new bare repository on the remote host.
pub fn create_repo(name: &str) -> Result<(), String> {
    validate_repo_name(name)?;

    // Paths are single-quoted; validate_repo_name already prohibits single quotes
    // and other shell-special characters, providing defence in depth.
    let path = repo_path(name);
    let script = format!("mkdir -p '{REPOS_DIR}' && git init --bare '{path}'");

    let output = Command::new("ssh")
        .args([SSH_HOST, &script])
        .output()
        .map_err(|e| format!("Failed to run ssh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create repository: {stderr}"));
    }

    Ok(())
}

/// Deletes a bare repository on the remote host.
pub fn delete_repo(name: &str) -> Result<(), String> {
    validate_repo_name(name)?;

    // Restrict deletion strictly to the repos directory; the validated name
    // cannot contain path separators or shell metacharacters.
    let path = repo_path(name);
    let script = format!("rm -rf '{REPOS_DIR}/{name}.git' && true");
    // Double-check the path stays inside REPOS_DIR (defense in depth).
    if !path.starts_with(REPOS_DIR) {
        return Err("Invalid repository path".to_string());
    }

    let output = Command::new("ssh")
        .args([SSH_HOST, &script])
        .output()
        .map_err(|e| format!("Failed to run ssh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete repository: {stderr}"));
    }

    Ok(())
}

/// Renames a bare repository on the remote host.
pub fn rename_repo(old_name: &str, new_name: &str) -> Result<(), String> {
    validate_repo_name(old_name)?;
    validate_repo_name(new_name)?;

    let old_path = repo_path(old_name);
    let new_path = repo_path(new_name);
    // Paths are single-quoted for additional shell safety.
    let script = format!("mv '{old_path}' '{new_path}'");

    let output = Command::new("ssh")
        .args([SSH_HOST, &script])
        .output()
        .map_err(|e| format!("Failed to run ssh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to rename repository: {stderr}"));
    }

    Ok(())
}
