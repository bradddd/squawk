use crate::github::{comment_on_pr, GithubError, PullRequest};
use crate::reporter::{check_files, get_comment_body, CheckFilesError};
use log::info;
use serde_json::Value;
use structopt::StructOpt;

#[derive(Debug)]
pub enum SquawkError {
    CheckFilesError(CheckFilesError),
    GithubError(GithubError),
    Base64DecodeError(base64::DecodeError),
    ByteDecodeError(std::string::FromUtf8Error),
    GithubPrivateKeyMissing,
}

impl std::convert::From<GithubError> for SquawkError {
    fn from(e: GithubError) -> Self {
        Self::GithubError(e)
    }
}

impl std::convert::From<CheckFilesError> for SquawkError {
    fn from(e: CheckFilesError) -> Self {
        Self::CheckFilesError(e)
    }
}

impl std::convert::From<base64::DecodeError> for SquawkError {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64DecodeError(e)
    }
}

impl std::convert::From<std::string::FromUtf8Error> for SquawkError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::ByteDecodeError(e)
    }
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Comment on a PR with Squawk's results.
    UploadToGithub {
        /// Paths to search
        paths: Vec<String>,
        /// Exclude specific warnings
        ///
        /// For example:
        /// --exclude=require-concurrent-index-creation,ban-drop-database
        #[structopt(short, long, use_delimiter = true)]
        exclude: Option<Vec<String>>,
        #[structopt(long, env = "SQUAWK_GITHUB_PRIVATE_KEY")]
        github_private_key: Option<String>,
        #[structopt(long, env = "SQUAWK_GITHUB_PRIVATE_KEY_BASE64")]
        github_private_key_base64: Option<String>,
        /// GitHub App Id.
        #[structopt(long, env = "SQUAWK_GITHUB_APP_ID")]
        github_app_id: i64,
        /// GitHub Install Id. The installation that squawk is acting on.
        #[structopt(long, env = "SQUAWK_GITHUB_INSTALL_ID")]
        github_install_id: i64,
        /// GitHub Bot Name.
        #[structopt(long, env = "SQUAWK_GITHUB_BOT_NAME")]
        github_bot_name: String,
        /// GitHub Repo Owner
        /// github.com/sbdchd/squawk, sbdchd is the owner
        #[structopt(long, env = "SQUAWK_GITHUB_REPO_OWNER")]
        github_repo_owner: String,
        /// GitHub Repo Name
        /// github.com/sbdchd/squawk, squawk is the name
        #[structopt(long, env = "SQUAWK_GITHUB_REPO_NAME")]
        github_repo_name: String,
        /// GitHub Pull Request Number
        /// github.com/sbdchd/squawk/pull/10, 10 is the PR number
        #[structopt(long, env = "SQUAWK_GITHUB_PR_NUMBER")]
        github_pr_number: i64,
    },
}

fn get_github_private_key(
    github_private_key: Option<String>,
    github_private_key_base64: Option<String>,
) -> Result<String, SquawkError> {
    match github_private_key {
        Some(private_key) => Ok(private_key),
        None => {
            let key = github_private_key_base64.ok_or(SquawkError::GithubPrivateKeyMissing)?;
            let bytes = base64::decode(key)?;
            Ok(String::from_utf8(bytes)?)
        }
    }
}

pub fn check_and_comment_on_pr(
    cmd: Command,
    is_stdin: bool,
    stdin_path: Option<String>,
) -> Result<Value, SquawkError> {
    let Command::UploadToGithub {
        paths,
        exclude,
        github_private_key,
        github_app_id,
        github_install_id,
        github_bot_name,
        github_repo_owner,
        github_repo_name,
        github_pr_number,
        github_private_key_base64,
    } = cmd;
    info!("checking files");
    let violations = check_files(&paths, is_stdin, stdin_path, exclude)?;
    info!("generating github comment body");
    let comment_body = get_comment_body(violations);
    let pr = PullRequest {
        issue: github_pr_number,
        owner: github_repo_owner,
        repo: github_repo_name,
    };

    let gh_private_key = get_github_private_key(github_private_key, github_private_key_base64)?;

    info!("commenting on PR");
    Ok(comment_on_pr(
        &gh_private_key,
        github_app_id,
        github_install_id,
        &github_bot_name,
        pr,
        comment_body,
    )?)
}