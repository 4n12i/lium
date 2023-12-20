// Copyright 2023 The ChromiumOS Authors
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://developers.google.com/open-source/licenses/bsd

use anyhow::Result;
use argh::FromArgs;
use lium::config::Config;

#[derive(FromArgs, PartialEq, Debug)]
/// list Android branches
#[argh(subcommand, name = "branch")]
pub struct Args {
    #[argh(subcommand)]
    nested: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    List(ArgsList),
}

pub fn run(args: &Args) -> Result<()> {
    match &args.nested {
        SubCommand::List(args) => run_branch_list(args),
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// List up Android branches
#[argh(subcommand, name = "list")]
pub struct ArgsList {}

fn run_branch_list(_args: &ArgsList) -> Result<()> {
    let config = Config::read()?;
    let branches = config.android_branches();

    println!("{}", branches.join(" "));

    Ok(())
}
