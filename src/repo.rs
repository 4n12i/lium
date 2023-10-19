// Copyright 2023 The ChromiumOS Authors
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://developers.google.com/open-source/licenses/bsd

use std::env;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use ptyprocess::WaitStatus;
use regex_macro::regex;

use crate::config::Config;
use crate::util::shell_helpers::get_stdout;
use crate::util::shell_helpers::pty_launch_command_with_stdout_label_and_process;
use crate::util::shell_helpers::run_bash_command;

/// This tries to get ChromeOS checkout directory in the following order
/// 1. user specified directory via a given command line argument
/// 2. CROS_DIR environmental variables
/// 3. default_cros_checkout config setting
/// 4. current directory
pub fn get_cros_dir_unchecked(dir: &Option<String>) -> Result<String> {
    if let Some(crosdir) = dir {
        return Ok(crosdir.to_string());
    }

    if let Ok(crosdir) = env::var("CROS_DIR") {
        return Ok(crosdir);
    }

    if let Some(crosdir) = Config::read()?.default_cros_checkout() {
        return Ok(crosdir);
    }

    find_cros_dir_from_cwd()
}

pub fn get_repo_dir(dir: &Option<String>) -> Result<String> {
    let repo = get_cros_dir_unchecked(dir)?;
    ensure_is_cros_dir(&repo)?;
    Ok(repo)
}

pub fn get_current_synced_version(repo: &str) -> Result<String> {
    ensure_is_cros_dir(repo)?;

    let cmd = "./src/third_party/chromiumos-overlay/chromeos/config/chromeos_version.sh | grep -e \
               VERSION_STRING -e CHROME_BRANCH | cut -d '=' -f 2 | cut -d '-' -f 1";
    let output = run_bash_command(cmd, Some(repo))?;
    let binding = get_stdout(&output);
    let output: Vec<&str> = binding.split('\n').collect();
    let version = format!("R{}-{}", output[0], output[1]);
    let re_cros_version = regex!(r"R\d+\-\d+\.\d+\.\d+");
    if re_cros_version.is_match(&version) {
        Ok(version)
    } else {
        bail!("Invalid version format: {}", version)
    }
}

pub fn get_current_synced_arc_version(repo: &str) -> Result<String> {
    // TODO: Are there any better way to do?
    let cmd = "cd .repo/manifests && git branch -r --contains HEAD | xargs -n 1 | grep m/ | sed \
               -E 's@m/(.*)-arc@\\1@g'";
    let output = run_bash_command(cmd, Some(repo))?;
    Ok(get_stdout(&output))
}

pub fn repo_sync(repo: &str, force: bool) -> Result<()> {
    // TODO: Add --force option.
    println!("Running repo sync...");
    let cmd_result = pty_launch_command_with_stdout_label_and_process(
        "repo",
        repo,
        &["sync", "-j", &num_cpus::get().to_string()],
    )?;

    match cmd_result {
        WaitStatus::Exited(_pid, 0) => {
            println!("repo sync done!")
        }
        _ => {
            if force {
                println!("--force option not yet available.");
            }
        }
    }

    Ok(())
}

fn is_cros_dir(dir: &str) -> bool {
    let path = PathBuf::from(dir);
    path.is_dir() && path.join(".repo").is_dir() && path.join("chromite").join("bin").is_dir()
}

fn ensure_is_cros_dir(path: &str) -> Result<()> {
    if is_cros_dir(path) {
        return Ok(());
    }

    Err(anyhow!(
        "{path} is not a Chrom(e|ium) OS checkout. Please consider specifying --repo option."
    ))
}

fn find_cros_dir_from_cwd() -> Result<String> {
    let mut path = env::current_dir()?;
    let mut dir = path.to_string_lossy().to_string();

    while !is_cros_dir(&dir) {
        match path.parent() {
            Some(p) => path = p.to_path_buf(),
            None => bail!("Failed to find Cros SDK dir"),
        }
        dir = path.to_string_lossy().to_string();
    }
    Ok(dir)
}
