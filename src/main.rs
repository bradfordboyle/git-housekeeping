extern crate chrono;
extern crate csv;
extern crate failure;
extern crate git2;
#[macro_use]
extern crate structopt;

use std::io;

use chrono::{TimeZone, UTC};
use git2::{BranchType, Repository};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "git-housekeeping", about = "keep your repos tidy")]
enum Opt {
    #[structopt(name = "branches")]
    Branches { path: String },
}

fn branches(repo_name: &str) -> Result<(), failure::Error> {
    let repo = Repository::open(repo_name)?;
    let branches = repo.branches(Some(BranchType::Remote))?;

    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["author", "name", "when"])?;

    for b in branches {
        let (branch, _) = b?;
        let reference = branch.get();

        let shorthand = reference
            .shorthand()
            .ok_or(failure::err_msg("no reference_shorthand"))?;
        if !shorthand.starts_with("origin/") {
            continue;
        }
        let name = &shorthand[7..];

        let commit = reference.peel_to_commit()?;
        let author = commit.author();

        let author_name = author.name().ok_or(failure::err_msg("no author name"))?;
        let when = UTC.timestamp(author.when().seconds(), 0);

        wtr.write_record(&[author_name, name, &when.to_rfc3339()])?;
    }
    wtr.flush()?;

    Ok(())
}

fn main() -> Result<(), failure::Error> {
    let opt = Opt::from_args();

    match opt {
        Opt::Branches { path } => branches(&path),
    }
}
