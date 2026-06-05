use crate::HandlerResult;
use crate::codex_app_server_policy::{HookPolicyDecision, evaluate_hook_policy};
use crate::runtime_config::validate_runtime_config_file;
use std::collections::HashMap;
use std::process;

const ALLOW: i32 = 0;
const SKIP: i32 = 10;
const POLICY_ERROR: i32 = 20;
const CONFIG_PARSE_ERROR: i32 = 30;

pub fn runtime_policy_check(args: &[String]) -> HandlerResult {
    if args.len() != 1 {
        return Err("Usage: vibeguard-runtime runtime-policy-check <hook-name>".into());
    }

    let user_config = std::env::var("VIBEGUARD_USER_CONFIG_FILE").unwrap_or_default();
    if let Err(err) = validate_runtime_config_file(&user_config) {
        eprintln!("{}", err.message);
        process::exit(err.exit_code);
    }

    let decision = evaluate_hook_policy(&args[0], None, &HashMap::new());
    match decision {
        HookPolicyDecision::Run { reason, .. } => {
            if let Some(reason) = reason {
                println!("{reason}");
            }
            process::exit(ALLOW);
        }
        HookPolicyDecision::Skip(reason) => {
            println!("{reason}");
            process::exit(SKIP);
        }
        HookPolicyDecision::Error(reason) => {
            eprintln!("{reason}");
            process::exit(policy_error_exit_code(&reason));
        }
    }
}

fn policy_error_exit_code(reason: &str) -> i32 {
    if reason.contains("project config invalid JSON")
        || reason.contains("project config invalid UTF-8")
    {
        CONFIG_PARSE_ERROR
    } else {
        POLICY_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::policy_error_exit_code;

    #[test]
    fn runtime_policy_project_json_parse_errors_keep_config_parse_exit_code() {
        assert_eq!(
            policy_error_exit_code(
                "VibeGuard project config invalid JSON: /tmp/.vibeguard.json: EOF"
            ),
            30
        );
    }

    #[test]
    fn runtime_policy_project_utf8_parse_errors_keep_config_parse_exit_code() {
        assert_eq!(
            policy_error_exit_code("VibeGuard project config invalid UTF-8: /tmp/.vibeguard.json"),
            30
        );
    }

    #[test]
    fn runtime_policy_schema_errors_keep_policy_error_exit_code() {
        assert_eq!(
            policy_error_exit_code(
                "VibeGuard project config invalid: /tmp/.vibeguard.json disabled_hooks contains unsupported hook missing-hook"
            ),
            20
        );
    }
}
