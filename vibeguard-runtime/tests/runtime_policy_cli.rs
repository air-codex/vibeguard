use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vibeguard-runtime"))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "vibeguard-runtime-policy-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn write_policy(repo: &Path, body: &str) {
    fs::create_dir_all(repo).expect("repo temp dir should be created");
    fs::write(repo.join(".vibeguard.json"), body).expect("project policy should be written");
}

fn run_runtime_policy(repo: &Path, hook_name: &str) -> std::process::Output {
    bin()
        .arg("runtime-policy-check")
        .arg(hook_name)
        .current_dir(repo)
        .env_remove("VIBEGUARD_PROJECT_CONFIG")
        .env_remove("VIBEGUARD_USER_CONFIG_FILE")
        .output()
        .expect("runtime policy command should run")
}

#[test]
fn runtime_policy_check_allows_when_no_project_config_exists() {
    let repo = unique_temp_dir("allow_no_config");
    fs::create_dir_all(&repo).expect("repo temp dir should be created");

    let output = run_runtime_policy(&repo, "pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_skips_disabled_hook() {
    let repo = unique_temp_dir("disabled_hook");
    write_policy(&repo, r#"{"disabled_hooks":["pre-bash-guard"]}"#);

    let output = run_runtime_policy(&repo, "vibeguard-pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(10));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("disabled_hooks contains pre-bash-guard")
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_reports_warn_enforcement() {
    let repo = unique_temp_dir("warn_enforcement");
    write_policy(&repo, r#"{"enforcement":"warn"}"#);

    let output = run_runtime_policy(&repo, "pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("enforcement=warn"));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_validates_user_runtime_config_before_policy() {
    let repo = unique_temp_dir("bad_user_config");
    write_policy(&repo, r#"{}"#);
    let user_config = repo.join("bad-config.json");
    fs::write(&user_config, r#"{"write_mode":"#).expect("runtime config should be written");

    let output = bin()
        .arg("runtime-policy-check")
        .arg("pre-bash-guard.sh")
        .current_dir(&repo)
        .env_remove("VIBEGUARD_PROJECT_CONFIG")
        .env("VIBEGUARD_USER_CONFIG_FILE", &user_config)
        .output()
        .expect("runtime policy command should run");

    assert_eq!(output.status.code(), Some(30));
    assert!(String::from_utf8_lossy(&output.stderr).contains("runtime config invalid JSON"));
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_reports_project_schema_errors_as_policy_errors() {
    let repo = unique_temp_dir("bad_project_schema");
    write_policy(&repo, r#"{"disabled_hooks":["missing-hook"]}"#);

    let output = run_runtime_policy(&repo, "pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(20));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("disabled_hooks contains unsupported hook")
    );
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_reports_project_json_parse_errors_as_config_parse_errors() {
    let repo = unique_temp_dir("bad_project_json");
    write_policy(&repo, r#"{"disabled_hooks":"#);

    let output = run_runtime_policy(&repo, "pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(30));
    assert!(String::from_utf8_lossy(&output.stderr).contains("project config invalid JSON"));
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_reports_project_utf8_errors_as_config_parse_errors() {
    let repo = unique_temp_dir("bad_project_utf8");
    fs::create_dir_all(&repo).expect("repo temp dir should be created");
    fs::write(repo.join(".vibeguard.json"), [0xff, 0xfe])
        .expect("invalid utf8 project policy should be written");

    let output = run_runtime_policy(&repo, "pre-bash-guard.sh");

    assert_eq!(output.status.code(), Some(30));
    assert!(String::from_utf8_lossy(&output.stderr).contains("project config invalid UTF-8"));
    let _ = fs::remove_dir_all(repo);
}

#[test]
fn runtime_policy_check_uses_shared_profile_filtering_for_strict_only_hooks() {
    let repo = unique_temp_dir("strict_only");
    write_policy(&repo, r#"{"profile":"core"}"#);

    let output = run_runtime_policy(&repo, "count_active_constraints.sh");

    assert_eq!(output.status.code(), Some(10));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("profile=core excludes count-active-constraints")
    );
    let _ = fs::remove_dir_all(repo);
}
