//! A project's sources through the `git` command line: one clone per
//! project under the data directory, checked out at the commit a build
//! needs.

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::application::ports::{Checkout, Sources};
use crate::domain::build::CommitId;
use crate::domain::machine::Tool;
use crate::domain::recipe::{Patch, PatchRefused, Recipe};

use super::command::{self, Output};
use super::files::remove_dir_if_present;

/// The clones live under `root`, one directory per project.
pub struct GitSources {
    root: PathBuf,
}

impl GitSources {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Where the project's clone lives.
    pub fn clone_dir(&self, recipe: &Recipe) -> PathBuf {
        self.root.join(recipe.project)
    }

    /// Clones the repository with its history but no file contents, which
    /// git fetches when a commit is checked out. The clone is made beside
    /// its final place and renamed, so an interrupted clone leaves no
    /// half-made repository behind.
    fn clone(&self, recipe: &Recipe, dir: &Path) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("cannot create {}", self.root.display()))?;
        let partial = dir.with_extension("partial");
        remove_dir_if_present(&partial)?;
        let clone = git(
            None,
            [
                OsStr::new("clone"),
                OsStr::new("--quiet"),
                OsStr::new("--filter=blob:none"),
                OsStr::new("--no-checkout"),
                OsStr::new("--"),
                OsStr::new(recipe.repository),
                partial.as_os_str(),
            ],
        );
        let cloned = run(clone).with_context(|| format!("cannot clone {}", recipe.repository));
        if let Err(err) = cloned {
            // Best effort: the clone error is what matters.
            let _ = fs::remove_dir_all(&partial);
            return Err(err);
        }
        fs::rename(&partial, dir)
            .with_context(|| format!("cannot move the clone to {}", dir.display()))
    }

    /// Updates an existing clone: branches, and tags even when moved. A tag
    /// the remote no longer holds, such as a release its project withdrew,
    /// leaves the clone: `--prune` alone keeps the tags `--tags` fetched.
    fn update(&self, recipe: &Recipe, dir: &Path) -> Result<()> {
        let origin = run(git(Some(dir), ["remote", "get-url", "origin"]))
            .with_context(|| format!("{} is not a usable clone", dir.display()))?;
        let origin = origin.stdout.trim();
        ensure!(
            origin == recipe.repository,
            "{} is a clone of {origin}, not of {}; remove it and ai-ctl clones the right one",
            dir.display(),
            recipe.repository
        );
        run(git(
            Some(dir),
            [
                "fetch",
                "--quiet",
                "--prune",
                "--prune-tags",
                "--tags",
                "--force",
                "origin",
            ],
        ))
        .with_context(|| format!("cannot fetch {}", recipe.repository))?;
        Ok(())
    }

    /// The commit `reference` names: a tag, a branch of the remote, or a
    /// commit, in that order. The partial clone fetches by itself a commit
    /// it does not have, such as one no branch holds, when `rev-parse` asks
    /// for its full hash.
    fn resolve(&self, recipe: &Recipe, dir: &Path, reference: &str) -> Result<CommitId> {
        let candidates = [
            format!("refs/tags/{reference}"),
            format!("refs/remotes/origin/{reference}"),
            reference.to_owned(),
        ];
        for candidate in &candidates {
            if let Some(commit) = rev_parse(dir, candidate)? {
                return Ok(commit);
            }
        }
        bail!(
            "`{reference}` is not a tag, a branch or a commit of {}",
            recipe.repository
        )
    }
}

impl Sources for GitSources {
    fn fetch(&self, recipe: &Recipe) -> Result<()> {
        let dir = self.clone_dir(recipe);
        if dir.join(".git").is_dir() {
            return self.update(recipe, &dir);
        }
        ensure!(
            !dir.exists(),
            "{} exists but is not a git clone; remove it and ai-ctl clones {} again",
            dir.display(),
            recipe.repository
        );
        self.clone(recipe, &dir)
    }

    fn release_tags(&self, recipe: &Recipe) -> Result<Vec<String>> {
        let dir = self.clone_dir(recipe);
        let output = run(git(Some(&dir), ["tag", "--list", &recipe.tags.glob()]))
            .with_context(|| format!("cannot list the tags of {}", dir.display()))?;
        Ok(output.stdout.lines().map(str::to_owned).collect())
    }

    fn remote_tags(&self, recipe: &Recipe) -> Result<Vec<String>> {
        let glob = recipe.tags.glob();
        // Out of any repository: one around the working directory would
        // lend the listing its configuration.
        let listing = git(
            Some(Path::new("/")),
            [
                "ls-remote",
                "--tags",
                "--refs",
                "--",
                recipe.repository,
                glob.as_str(),
            ],
        );
        let listing = run(listing)
            .with_context(|| format!("cannot read the tags of {}", recipe.repository))?;
        Ok(remote_tag_names(&listing.stdout))
    }

    fn checkout(&self, recipe: &Recipe, reference: &str) -> Result<Checkout> {
        ensure!(
            !reference.is_empty() && !reference.starts_with('-'),
            "`{reference}` is not a git reference"
        );
        let dir = self.clone_dir(recipe);
        ensure!(
            dir.join(".git").is_dir(),
            "no clone of {} in {}",
            recipe.repository,
            dir.display()
        );
        let commit = self.resolve(recipe, &dir, reference)?;
        run(git(
            Some(&dir),
            [
                "checkout",
                "--quiet",
                "--force",
                "--detach",
                commit.as_str(),
            ],
        ))
        .with_context(|| format!("cannot check out {commit}"))?;
        // Files a previous build or a person left in the tree would end up
        // in the build.
        run(git(Some(&dir), ["clean", "-ffdxq"]))
            .with_context(|| format!("cannot clean {}", dir.display()))?;
        update_submodules(recipe, &dir)?;
        let tags = run(git(Some(&dir), ["tag", "--points-at", commit.as_str()]))
            .context("cannot list the tags of the commit")?;
        Ok(Checkout {
            dir,
            commit,
            tags: tags.stdout.lines().map(str::to_owned).collect(),
        })
    }

    fn apply(&self, checkout: &Checkout, patch: &Patch) -> Result<()> {
        // `git apply` reads a file, kept in the clone's git directory, out
        // of the tree it changes.
        let file = checkout
            .dir
            .join(".git")
            .join(format!("ai-ctl-{}", patch.name));
        fs::write(&file, patch.diff).with_context(|| format!("cannot write {}", file.display()))?;
        let applied = command::checked(
            command::execute(
                git(
                    Some(&checkout.dir),
                    [OsStr::new("apply"), OsStr::new("--"), file.as_os_str()],
                ),
                None,
            ),
            &missing(),
        );
        // Best effort: the patch's outcome is what matters.
        let _ = fs::remove_file(&file);
        // git ran: a failure is its refusal of the patch.
        let output = applied?;
        if !output.success {
            return Err(PatchRefused(output.failure()).into());
        }
        Ok(())
    }
}

/// Checks out the recipe's submodules at the commits the checkout names,
/// fetched as the clone is, and leaves in them only what that commit holds.
/// Their URLs follow the checked-out `.gitmodules`, which a release may
/// change. A submodule of a submodule, named by its whole path, is checked
/// out within its parent once the parent is, since git takes from a
/// repository only its own submodules; one the release does not hold is
/// skipped.
fn update_submodules(recipe: &Recipe, dir: &Path) -> Result<()> {
    let named = recipe.submodules;
    let mut parents: Vec<&str> = std::iter::once("").chain(named.iter().copied()).collect();
    parents.sort_by_key(|parent| parent.split('/').filter(|part| !part.is_empty()).count());
    let mut checked_out = vec![""];
    for parent in parents {
        if !checked_out.contains(&parent) {
            continue;
        }
        let children: Vec<&str> = named
            .iter()
            .copied()
            .filter(|path| parent_of(named, path) == parent)
            .collect();
        checked_out.extend(update_within(dir, parent, &children)?);
    }
    Ok(())
}

/// The named submodule `path` lies in, or `""` for the clone itself.
fn parent_of<'a>(named: &[&'a str], path: &str) -> &'a str {
    named
        .iter()
        .copied()
        .filter(|parent| {
            path.strip_prefix(parent)
                .is_some_and(|rest| rest.starts_with('/'))
        })
        .max_by_key(|parent| parent.len())
        .unwrap_or("")
}

/// Checks out `paths`, the named submodules of the repository at `parent`,
/// and returns those its commit holds.
fn update_within<'a>(dir: &Path, parent: &str, paths: &[&'a str]) -> Result<Vec<&'a str>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let repository = dir.join(parent);
    // Each path starts with `parent/`, as `parent_of` found it.
    let relative = |path: &'a str| -> &'a str {
        if parent.is_empty() {
            path
        } else {
            &path[parent.len() + 1..]
        }
    };
    let relatives: Vec<&str> = paths.iter().map(|path| relative(path)).collect();
    let staged = run(git(
        Some(&repository),
        [&["ls-files", "--stage", "-z", "--"][..], &relatives].concat(),
    ))
    .with_context(|| format!("cannot list the submodules of {}", repository.display()))?;
    // A submodule is a `160000` entry; a release may lack one a recipe names.
    let held: Vec<&str> = paths
        .iter()
        .copied()
        .filter(|path| {
            staged.stdout.split('\0').any(|entry| {
                entry.split_once('\t').is_some_and(|(mode, staged)| {
                    mode.starts_with("160000 ") && staged == relative(path)
                })
            })
        })
        .collect();
    // Without a path, `git submodule update --init` takes every submodule.
    if held.is_empty() {
        return Ok(held);
    }
    let relatives: Vec<&str> = held.iter().map(|path| relative(path)).collect();
    run(git(
        Some(&repository),
        [&["submodule", "--quiet", "sync", "--"][..], &relatives].concat(),
    ))
    .context("cannot set the submodules' URLs")?;
    let update = [
        &["submodule", "--quiet", "update", "--init", "--force"][..],
        &["--filter=blob:none", "--"],
        &relatives,
    ]
    .concat();
    let updated = run(git(Some(&repository), &update));
    // A clone killed during its first fetch leaves a git directory with no
    // commit, which git takes as cloned: removed, it is cloned again.
    let updated = match updated {
        Err(err) if remove_unborn(&repository, &relatives).unwrap_or(false) => {
            run(git(Some(&repository), &update)).map_err(|again| again.context(err))
        }
        updated => updated,
    };
    updated.with_context(|| format!("cannot check out the submodules {}", held.join(", ")))?;
    for path in &held {
        let submodule = dir.join(path);
        run(git(Some(&submodule), ["clean", "-ffdxq"]))
            .with_context(|| format!("cannot clean {}", submodule.display()))?;
    }
    Ok(held)
}

/// Removes the git directory, and the path, of each submodule `relatives`
/// of the repository at `repository` whose git directory holds no commit:
/// whether one was removed.
fn remove_unborn(repository: &Path, relatives: &[&str]) -> Result<bool> {
    let git_dir = run(git(Some(repository), ["rev-parse", "--absolute-git-dir"]))?;
    let modules = Path::new(git_dir.stdout.trim()).join("modules");
    let paths = run(git(
        Some(repository),
        [
            "config",
            "--null",
            "--file",
            ".gitmodules",
            "--get-regexp",
            r"^submodule\..*\.path$",
        ],
    ))?;
    let mut removed = false;
    for (name, path) in submodule_paths(&paths.stdout) {
        let module = modules.join(name);
        if !relatives.contains(&path)
            || !module.join("HEAD").is_file()
            || rev_parse(&module, "HEAD")?.is_some()
        {
            continue;
        }
        remove_dir_if_present(&module)?;
        remove_dir_if_present(&repository.join(path))?;
        removed = true;
    }
    Ok(removed)
}

/// Each submodule's name and path in `listing`, the output of `git config
/// --null --get-regexp` on `.gitmodules`' paths: entries `<key>\n<value>\0`,
/// so that a name may hold a space. A name comes from upstream's
/// `.gitmodules`: one that would climb out of `modules`, which git itself
/// ignores, is left out.
fn submodule_paths(listing: &str) -> Vec<(&str, &str)> {
    listing
        .split('\0')
        .filter_map(|entry| {
            let (key, path) = entry.split_once('\n')?;
            let name = key.strip_prefix("submodule.")?.strip_suffix(".path")?;
            Path::new(name)
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
                .then_some((name, path))
        })
        .collect()
}

/// The tags in `listing`, the output of `git ls-remote --tags --refs`: one
/// line `<hash><TAB>refs/tags/<tag>` each.
fn remote_tag_names(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| line.split_once('\t')?.1.strip_prefix("refs/tags/"))
        .map(str::to_owned)
        .collect()
}

/// A `git` command in `dir`. Git never waits for a password, and a transfer
/// slower than 1 KB/s for a minute is aborted rather than left hanging.
fn git<I, S>(dir: Option<&Path>, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    // The tests' submodules are local repositories, which git clones for a
    // submodule only when told to.
    #[cfg(test)]
    command.args(["-c", "protocol.file.allow=always"]);
    command
        .args([
            "-c",
            "color.ui=never",
            "-c",
            "http.lowSpeedLimit=1000",
            "-c",
            "http.lowSpeedTime=60",
        ])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    command
}

/// Runs a git command; its first error line explains a failure.
fn run(command: Command) -> Result<Output> {
    let output = command::checked(command::execute(command, None), &missing())?;
    ensure!(output.success, output.failure());
    Ok(output)
}

/// The error of a machine without git.
fn missing() -> String {
    command::not_found(Tool::Git)
}

/// The commit `revision` names in the clone, or `None` when it names none.
fn rev_parse(dir: &Path, revision: &str) -> Result<Option<CommitId>> {
    let peeled = format!("{revision}^{{commit}}");
    let output = command::execute(
        git(Some(dir), ["rev-parse", "--verify", "--quiet", &peeled]),
        None,
    );
    let output = command::checked(output, &missing())?;
    // `--quiet`: an unknown revision fails without a message.
    if !output.success {
        return Ok(None);
    }
    let hash = output.stdout.trim();
    CommitId::parse(hash)
        .map(Some)
        .with_context(|| format!("unexpected `git rev-parse` output for {revision}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::recipe::{
        AUDIO_CPP, LLAMA_CPP, STABLE_DIFFUSION_CPP, TagPattern, WHISPER_CPP,
    };

    /// A repository standing for the project's remote, with releases `b1`,
    /// `b2` and `b10` on `master`, tags that are not releases, and a
    /// `feature` branch one commit ahead.
    struct Upstream {
        dir: tempfile::TempDir,
        url: String,
        commits: Vec<String>,
    }

    /// Runs git with neither the user's nor the system's configuration.
    fn plain_git(dir: &Path, args: &[&str]) -> String {
        let mut git = Command::new("git");
        git.arg("-C")
            .arg(dir)
            .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let output = command::spawn(&mut git)
            .unwrap()
            .wait_with_output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn upstream() -> Upstream {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        plain_git(repo, &["init", "--quiet", "--initial-branch=master"]);
        plain_git(repo, &["config", "uploadpack.allowFilter", "true"]);
        plain_git(repo, &["config", "uploadpack.allowAnySHA1InWant", "true"]);
        let mut commits = Vec::new();
        for (i, tag) in ["b1", "b2", "b10"].into_iter().enumerate() {
            fs::write(repo.join("version.txt"), format!("{i}\n")).unwrap();
            plain_git(repo, &["add", "version.txt"]);
            plain_git(repo, &["commit", "--quiet", "-m", tag]);
            plain_git(repo, &["tag", tag]);
            commits.push(plain_git(repo, &["rev-parse", "HEAD"]));
        }
        plain_git(repo, &["tag", "-a", "-m", "not a release", "b10-rc"]);
        plain_git(repo, &["tag", "backup/old", &commits[0]]);
        plain_git(repo, &["checkout", "--quiet", "-b", "feature"]);
        fs::write(repo.join("feature.txt"), "new\n").unwrap();
        plain_git(repo, &["add", "feature.txt"]);
        plain_git(repo, &["commit", "--quiet", "-m", "feature"]);
        commits.push(plain_git(repo, &["rev-parse", "HEAD"]));
        plain_git(repo, &["checkout", "--quiet", "master"]);
        Upstream {
            url: format!("file://{}", repo.display()),
            dir,
            commits,
        }
    }

    fn recipe(url: &str) -> Recipe {
        Recipe {
            project: "demo",
            repository: Box::leak(url.to_owned().into_boxed_str()),
            tags: TagPattern::Numbered { prefix: "b" },
            ..LLAMA_CPP
        }
    }

    #[test]
    fn each_captured_listing_of_the_remote_gives_its_tags_and_newest_release() {
        // `git ls-remote --tags --refs` of each repository, on 2026-11-09.
        let listings = [
            (
                LLAMA_CPP,
                include_str!("../../tests/fixtures/llama.cpp-git-ls-remote-excerpt.txt"),
                58,
                57,
                "b12226",
            ),
            (
                WHISPER_CPP,
                include_str!("../../tests/fixtures/whisper.cpp-git-ls-remote.txt"),
                44,
                40,
                "v1.9.5",
            ),
            (
                STABLE_DIFFUSION_CPP,
                include_str!("../../tests/fixtures/stable-diffusion.cpp-git-ls-remote.txt"),
                733,
                603,
                "master-1068-fc350be",
            ),
            (
                AUDIO_CPP,
                include_str!("../../tests/fixtures/audio.cpp-git-ls-remote.txt"),
                18,
                13,
                "v0.9.4",
            ),
        ];
        for (recipe, listing, listed, releases, newest) in listings {
            let tags = remote_tag_names(listing);
            assert_eq!(tags.len(), listed, "{newest}");
            assert!(tags.iter().all(|tag| !tag.contains("refs/")), "{newest}");
            let found = tags.iter().filter(|tag| recipe.tags.release(tag).is_some());
            assert_eq!(found.count(), releases, "{newest}");
            assert_eq!(
                recipe.tags.newest(tags.iter().map(String::as_str)),
                Some(newest)
            );
        }
    }

    #[test]
    fn a_listing_gives_tags_alone() {
        assert_eq!(
            remote_tag_names(
                "aaa\trefs/heads/master\nbbb\trefs/tags/b2\n\nno tab\nccc\trefs/tags/v1/b3\n"
            ),
            ["b2", "v1/b3"]
        );
        assert!(remote_tag_names("").is_empty());
    }

    #[test]
    fn remote_tags_need_no_clone_and_write_nothing() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().join("sources"));
        let recipe = recipe(&upstream.url);
        let mut tags = sources.remote_tags(&recipe).unwrap();
        tags.sort();
        // The annotated tag once, with no peeled line; no tag of another
        // form, and no branch.
        assert_eq!(tags, ["b1", "b10", "b10-rc", "b2"]);
        assert!(!root.path().join("sources").exists());
    }

    #[test]
    fn a_remote_that_cannot_be_read_names_its_repository() {
        let root = tempfile::tempdir().unwrap();
        let gone = root.path().join("gone");
        let recipe = recipe(&format!("file://{}", gone.display()));
        let sources = GitSources::new(root.path().join("sources"));
        let error = sources.remote_tags(&recipe).unwrap_err();
        let message = format!("{error:#}");
        assert!(
            message.starts_with(&format!(
                "cannot read the tags of file://{}",
                gone.display()
            )),
            "{message}"
        );
    }

    #[test]
    fn release_tags_are_read_from_the_clone() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();
        drop(upstream);
        let mut tags = sources.release_tags(&recipe).unwrap();
        tags.sort();
        assert_eq!(tags, ["b1", "b10", "b10-rc", "b2"]);
        assert_eq!(
            recipe.tags.newest(tags.iter().map(String::as_str)),
            Some("b10")
        );
    }

    #[test]
    fn the_first_fetch_clones_then_later_ones_update() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().join("sources"));
        let recipe = recipe(&upstream.url);

        sources.fetch(&recipe).unwrap();
        let first = sources.checkout(&recipe, "b2").unwrap();
        assert_eq!(first.dir, root.path().join("sources/demo"));
        assert_eq!(first.commit.as_str(), upstream.commits[1]);
        assert_eq!(first.tags, ["b2"]);
        assert_eq!(
            fs::read_to_string(first.dir.join("version.txt")).unwrap(),
            "1\n"
        );
        assert!(!root.path().join("sources/demo.partial").exists());

        // A stray file disappears; a branch and a full hash resolve.
        fs::write(first.dir.join("stray.o"), "").unwrap();
        sources.fetch(&recipe).unwrap();
        let feature = sources.checkout(&recipe, "feature").unwrap();
        assert_eq!(feature.commit.as_str(), upstream.commits[3]);
        assert!(feature.tags.is_empty());
        assert!(!feature.dir.join("stray.o").exists());
        assert!(feature.dir.join("feature.txt").is_file());

        let by_hash = sources.checkout(&recipe, &upstream.commits[0]).unwrap();
        assert_eq!(by_hash.tags, ["b1", "backup/old"]);
        let short = sources
            .checkout(&recipe, &upstream.commits[2][..8])
            .unwrap();
        assert_eq!(short.commit.as_str(), upstream.commits[2]);
        let mut tags = short.tags;
        tags.sort();
        assert_eq!(tags, ["b10", "b10-rc"]);
    }

    #[test]
    fn a_tag_the_remote_withdrew_leaves_the_clone_at_the_next_fetch() {
        let upstream = upstream();
        let repo = upstream.dir.path();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();

        // The newest release withdrawn, a tag that is not a release too.
        plain_git(repo, &["tag", "-d", "b10"]);
        plain_git(repo, &["tag", "-d", "backup/old"]);
        sources.fetch(&recipe).unwrap();

        let mut tags = sources.release_tags(&recipe).unwrap();
        tags.sort();
        assert_eq!(tags, ["b1", "b10-rc", "b2"]);
        assert_eq!(
            recipe.tags.newest(tags.iter().map(String::as_str)),
            Some("b2")
        );
        let message = format!("{:#}", sources.checkout(&recipe, "backup/old").unwrap_err());
        assert!(message.contains("`backup/old` is not a tag"), "{message}");
    }

    #[test]
    fn a_commit_no_branch_holds_is_found_by_its_hash() {
        let upstream = upstream();
        let repo = upstream.dir.path();
        plain_git(repo, &["checkout", "--quiet", "-b", "gone"]);
        fs::write(repo.join("gone.txt"), "gone\n").unwrap();
        plain_git(repo, &["add", "gone.txt"]);
        plain_git(repo, &["commit", "--quiet", "-m", "gone"]);
        let gone = plain_git(repo, &["rev-parse", "HEAD"]);
        plain_git(repo, &["checkout", "--quiet", "master"]);
        plain_git(repo, &["branch", "--quiet", "-D", "gone"]);
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);

        sources.fetch(&recipe).unwrap();
        let checkout = sources.checkout(&recipe, &gone).unwrap();
        assert_eq!(checkout.commit.as_str(), gone);
        assert!(checkout.dir.join("gone.txt").is_file());
    }

    #[test]
    fn a_checkout_needs_a_fetched_clone() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let message = format!(
            "{:#}",
            sources.checkout(&recipe(&upstream.url), "b1").unwrap_err()
        );
        assert_eq!(
            message,
            format!(
                "no clone of {} in {}",
                upstream.url,
                root.path().join("demo").display()
            )
        );
    }

    #[test]
    fn an_unknown_reference_is_refused() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();
        for reference in ["b99", "no-such-branch", &"0".repeat(40)] {
            let message = format!("{:#}", sources.checkout(&recipe, reference).unwrap_err());
            assert!(
                message.contains("is not a tag, a branch or a commit"),
                "{message}"
            );
        }
        for reference in ["", "--upload-pack=touch"] {
            let message = format!("{:#}", sources.checkout(&recipe, reference).unwrap_err());
            assert!(message.contains("is not a git reference"), "{message}");
        }
    }

    #[test]
    fn a_clone_of_another_repository_is_refused() {
        let first = upstream();
        let other = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        sources.fetch(&recipe(&first.url)).unwrap();
        let message = format!("{:#}", sources.fetch(&recipe(&other.url)).unwrap_err());
        assert!(message.contains("remove it"), "{message}");
    }

    #[test]
    fn a_failed_clone_leaves_nothing_behind() {
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let missing = root.path().join("missing");
        let recipe = recipe(&format!("file://{}", missing.display()));
        let message = format!("{:#}", sources.fetch(&recipe).unwrap_err());
        assert!(message.starts_with("cannot clone"), "{message}");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_directory_that_is_not_a_clone_is_left_alone() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("demo")).unwrap();
        fs::write(root.path().join("demo/notes.txt"), "mine").unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let message = format!("{:#}", sources.fetch(&recipe(&upstream.url)).unwrap_err());
        assert!(message.contains("is not a git clone"), "{message}");
        assert!(root.path().join("demo/notes.txt").is_file());
    }

    #[test]
    fn a_fetch_that_fails_is_an_error() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();
        drop(upstream);
        let message = format!("{:#}", sources.fetch(&recipe).unwrap_err());
        assert!(
            message.starts_with(&format!("cannot fetch {}", recipe.repository)),
            "{message}"
        );
    }

    /// A repository with two submodules, `ggml` and `frontend`: its
    /// releases `b1` and `b2` take `ggml` from one commit to the next, and
    /// `b3` to a commit of a fork.
    fn superproject() -> Upstream {
        let dir = tempfile::tempdir().unwrap();
        let repo = |name: &str| dir.path().join(name);
        let url = |name: &str| format!("file://{}", repo(name).display());
        for name in ["ggml", "frontend", "project"] {
            plain_git(
                dir.path(),
                &["init", "--quiet", "--initial-branch=master", name],
            );
        }
        let mut commits = Vec::new();
        for (name, content) in [("ggml", "1"), ("ggml", "2"), ("frontend", "f")] {
            fs::write(repo(name).join("version.txt"), format!("{content}\n")).unwrap();
            plain_git(&repo(name), &["add", "version.txt"]);
            plain_git(&repo(name), &["commit", "--quiet", "-m", content]);
            commits.push(plain_git(&repo(name), &["rev-parse", "HEAD"]));
        }
        plain_git(dir.path(), &["clone", "--quiet", &url("ggml"), "fork"]);
        fs::write(repo("fork").join("version.txt"), "3\n").unwrap();
        plain_git(&repo("fork"), &["commit", "--quiet", "-am", "3"]);
        for name in ["ggml", "fork", "frontend", "project"] {
            plain_git(&repo(name), &["config", "uploadpack.allowFilter", "true"]);
            plain_git(
                &repo(name),
                &["config", "uploadpack.allowAnySHA1InWant", "true"],
            );
        }

        let project = repo("project");
        let ggml = project.join("ggml");
        for name in ["ggml", "frontend"] {
            let add = [
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "--quiet",
            ];
            plain_git(&project, &[&add[..], &[&url(name), name]].concat());
        }
        plain_git(&ggml, &["checkout", "--quiet", &commits[0]]);
        plain_git(&project, &["commit", "--quiet", "-am", "b1"]);
        plain_git(&project, &["tag", "b1"]);
        plain_git(&ggml, &["checkout", "--quiet", &commits[1]]);
        plain_git(&project, &["commit", "--quiet", "-am", "b2"]);
        plain_git(&project, &["tag", "b2"]);
        plain_git(
            &project,
            &["submodule", "--quiet", "set-url", "ggml", &url("fork")],
        );
        plain_git(&ggml, &["fetch", "--quiet", &url("fork"), "master"]);
        plain_git(&ggml, &["checkout", "--quiet", "FETCH_HEAD"]);
        plain_git(&project, &["commit", "--quiet", "-am", "b3"]);
        plain_git(&project, &["tag", "b3"]);
        Upstream {
            url: url("project"),
            dir,
            commits,
        }
    }

    /// The content of `version.txt` in `dir`, when there is one.
    fn version(dir: &Path) -> Option<String> {
        fs::read_to_string(dir.join("version.txt")).ok()
    }

    #[test]
    fn a_recipes_submodules_are_checked_out_with_the_sources() {
        let upstream = superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        // A recipe that names no submodule gets none.
        let without = recipe(&upstream.url);
        sources.fetch(&without).unwrap();
        let checkout = sources.checkout(&without, "b1").unwrap();
        let ggml = checkout.dir.join("ggml");
        let frontend = checkout.dir.join("frontend");
        assert_eq!(fs::read_dir(&ggml).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&frontend).unwrap().count(), 0);

        let recipe = Recipe {
            submodules: &["ggml"],
            ..without
        };
        sources.checkout(&recipe, "b1").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("1\n"));
        // Fetched as the clone is: file contents only when checked out.
        assert_eq!(
            plain_git(&ggml, &["config", "remote.origin.partialclonefilter"]),
            "blob:none"
        );
        assert_eq!(fs::read_dir(&frontend).unwrap().count(), 0);

        // What a build or a person left in the submodule disappears.
        fs::write(ggml.join("version.txt"), "edited\n").unwrap();
        fs::write(ggml.join("stray.o"), "").unwrap();
        sources.checkout(&recipe, "b1").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("1\n"));
        assert!(!ggml.join("stray.o").exists());

        sources.checkout(&recipe, "b2").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("2\n"));
        // A release whose submodule moved to another repository.
        sources.checkout(&recipe, "b3").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("3\n"));
        sources.checkout(&recipe, "b1").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("1\n"));
        assert_eq!(fs::read_dir(&frontend).unwrap().count(), 0);

        // A submodule the release lacks is skipped, beside one it holds.
        let with_absent = Recipe {
            submodules: &["ggml", "webm"],
            ..recipe
        };
        let checked_out = sources
            .checkout(&with_absent, "b2")
            .map(drop)
            .map_err(|error| format!("{error:#}"));
        assert_eq!(checked_out, Ok(()));
        assert_eq!(version(&ggml).as_deref(), Some("2\n"));
    }

    #[test]
    fn a_submodule_whose_first_clone_was_killed_is_checked_out_next_time() {
        let upstream = superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = Recipe {
            submodules: &["ggml"],
            ..recipe(&upstream.url)
        };
        sources.fetch(&recipe).unwrap();
        let clone = sources.clone_dir(&recipe);
        let ggml_url = format!("file://{}", upstream.dir.path().join("ggml").display());
        killed_in_its_first_clone(&clone, "ggml", &ggml_url);
        let checked_out = sources
            .checkout(&recipe, "b1")
            .map(drop)
            .map_err(|error| format!("{error:#}"));
        assert_eq!(checked_out, Ok(()));
        assert_eq!(version(&clone.join("ggml")).as_deref(), Some("1\n"));
    }

    /// Leaves the submodule `name` of `clone` as a kill during its first
    /// clone leaves it: its git directory, configured, with no object or
    /// ref, and its gitfile.
    fn killed_in_its_first_clone(clone: &Path, name: &str, url: &str) {
        let modules = clone.join(".git/modules").join(name);
        fs::create_dir_all(clone.join(".git/modules")).unwrap();
        plain_git(
            clone,
            &[
                "init",
                "--quiet",
                &format!("--separate-git-dir={}", modules.display()),
                name,
            ],
        );
        plain_git(&clone.join(name), &["remote", "add", "origin", url]);
    }

    #[test]
    fn only_a_named_submodule_killed_in_its_first_clone_is_removed() {
        let upstream = superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = Recipe {
            submodules: &["ggml"],
            ..recipe(&upstream.url)
        };
        sources.fetch(&recipe).unwrap();
        let clone = sources.clone_dir(&recipe);
        for name in ["ggml", "frontend"] {
            let url = format!("file://{}", upstream.dir.path().join(name).display());
            killed_in_its_first_clone(&clone, name, &url);
        }
        sources.checkout(&recipe, "b1").unwrap();
        assert_eq!(version(&clone.join("ggml")).as_deref(), Some("1\n"));
        assert!(clone.join(".git/modules/frontend/HEAD").is_file());
    }

    #[test]
    fn a_submodule_with_commits_is_kept_when_its_update_fails() {
        let upstream = superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = Recipe {
            submodules: &["ggml"],
            ..recipe(&upstream.url)
        };
        sources.fetch(&recipe).unwrap();
        sources.checkout(&recipe, "b1").unwrap();
        // b2's file is fetched on demand, from an upstream now gone.
        fs::remove_dir_all(upstream.dir.path().join("ggml")).unwrap();
        let message = format!("{:#}", sources.checkout(&recipe, "b2").unwrap_err());
        assert!(
            message.starts_with("cannot check out the submodules ggml"),
            "{message}"
        );
        let modules = sources.clone_dir(&recipe).join(".git/modules/ggml");
        assert!(modules.join("HEAD").is_file(), "{message}");
    }

    /// A repository whose submodule `ggml` gains a submodule of its own,
    /// `vendors/cutlass`, between its releases `b1` and `b2`.
    fn nested_superproject() -> Upstream {
        let dir = tempfile::tempdir().unwrap();
        let repo = |name: &str| dir.path().join(name);
        let url = |name: &str| format!("file://{}", repo(name).display());
        let add = [
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "--quiet",
        ];
        for name in ["cutlass", "ggml", "project"] {
            plain_git(
                dir.path(),
                &["init", "--quiet", "--initial-branch=master", name],
            );
            plain_git(&repo(name), &["config", "uploadpack.allowFilter", "true"]);
            plain_git(
                &repo(name),
                &["config", "uploadpack.allowAnySHA1InWant", "true"],
            );
        }
        let mut commits = Vec::new();
        for (name, content) in [("cutlass", "c"), ("ggml", "1")] {
            fs::write(repo(name).join("version.txt"), format!("{content}\n")).unwrap();
            plain_git(&repo(name), &["add", "version.txt"]);
            plain_git(&repo(name), &["commit", "--quiet", "-m", content]);
            commits.push(plain_git(&repo(name), &["rev-parse", "HEAD"]));
        }
        plain_git(
            &repo("ggml"),
            &[&add[..], &[&url("cutlass"), "vendors/cutlass"]].concat(),
        );
        fs::write(repo("ggml").join("version.txt"), "2\n").unwrap();
        plain_git(&repo("ggml"), &["commit", "--quiet", "-am", "2"]);
        commits.push(plain_git(&repo("ggml"), &["rev-parse", "HEAD"]));

        let project = repo("project");
        let ggml = project.join("ggml");
        plain_git(&project, &[&add[..], &[&url("ggml"), "ggml"]].concat());
        plain_git(&ggml, &["checkout", "--quiet", &commits[1]]);
        plain_git(&project, &["commit", "--quiet", "-am", "b1"]);
        plain_git(&project, &["tag", "b1"]);
        plain_git(&ggml, &["checkout", "--quiet", &commits[2]]);
        plain_git(&project, &["commit", "--quiet", "-am", "b2"]);
        plain_git(&project, &["tag", "b2"]);
        Upstream {
            url: url("project"),
            dir,
            commits,
        }
    }

    #[test]
    fn a_submodule_of_a_submodule_is_checked_out_within_it() {
        let upstream = nested_superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let only_ggml = Recipe {
            submodules: &["ggml"],
            ..recipe(&upstream.url)
        };
        sources.fetch(&only_ggml).unwrap();
        let dir = sources.clone_dir(&only_ggml);
        let ggml = dir.join("ggml");
        let cutlass = ggml.join("vendors/cutlass");
        // A submodule's own submodules are not taken unless named.
        sources.checkout(&only_ggml, "b2").unwrap();
        assert_eq!(version(&ggml).as_deref(), Some("2\n"));
        assert_eq!(fs::read_dir(&cutlass).unwrap().count(), 0);

        let recipe = Recipe {
            submodules: &["ggml", "ggml/vendors/cutlass"],
            ..only_ggml
        };
        let checked_out = sources
            .checkout(&recipe, "b2")
            .map(drop)
            .map_err(|error| format!("{error:#}"));
        assert_eq!(checked_out, Ok(()));
        assert_eq!(version(&cutlass).as_deref(), Some("c\n"));
        assert_eq!(
            plain_git(&cutlass, &["config", "remote.origin.partialclonefilter"]),
            "blob:none"
        );

        // What a build or a person left in it disappears.
        fs::write(cutlass.join("version.txt"), "edited\n").unwrap();
        fs::write(cutlass.join("stray.o"), "").unwrap();
        sources.checkout(&recipe, "b2").unwrap();
        assert_eq!(version(&cutlass).as_deref(), Some("c\n"));
        assert!(!cutlass.join("stray.o").exists());

        // A release whose ggml has no such submodule is checked out without it.
        let checked_out = sources
            .checkout(&recipe, "b1")
            .map(drop)
            .map_err(|error| format!("{error:#}"));
        assert_eq!(checked_out, Ok(()));
        assert_eq!(version(&ggml).as_deref(), Some("1\n"));
        assert!(!cutlass.exists());
        sources.checkout(&recipe, "b2").unwrap();
        assert_eq!(version(&cutlass).as_deref(), Some("c\n"));
    }

    #[test]
    fn a_submodule_named_upstream_to_climb_out_is_left_out() {
        let listing = include_str!("../../tests/fixtures/git-config-null-submodule-paths.txt");
        assert_eq!(
            submodule_paths(listing),
            [("ggml", "ggml"), ("vendor stuff", "vendor stuff")]
        );
        assert!(submodule_paths("").is_empty());
    }

    /// A repository with a tracked file `version.txt` beside its submodules
    /// `a` and `a-extra`: `a` holds a submodule `b`, which holds `c`.
    fn deep_superproject() -> Upstream {
        let dir = tempfile::tempdir().unwrap();
        let repo = |name: &str| dir.path().join(name);
        let url = |name: &str| format!("file://{}", repo(name).display());
        let add = [
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "--quiet",
        ];
        for name in ["c", "b", "a", "a-extra", "project"] {
            plain_git(
                dir.path(),
                &["init", "--quiet", "--initial-branch=master", name],
            );
            plain_git(&repo(name), &["config", "uploadpack.allowFilter", "true"]);
            plain_git(
                &repo(name),
                &["config", "uploadpack.allowAnySHA1InWant", "true"],
            );
            fs::write(repo(name).join("version.txt"), format!("{name}\n")).unwrap();
            plain_git(&repo(name), &["add", "version.txt"]);
        }
        plain_git(&repo("c"), &["commit", "--quiet", "-m", "c"]);
        plain_git(&repo("a-extra"), &["commit", "--quiet", "-m", "a-extra"]);
        for (name, child) in [
            ("b", "c"),
            ("a", "b"),
            ("project", "a"),
            ("project", "a-extra"),
        ] {
            plain_git(&repo(name), &[&add[..], &[&url(child), child]].concat());
            if name != "project" {
                plain_git(&repo(name), &["commit", "--quiet", "-m", name]);
            }
        }
        plain_git(&repo("project"), &["commit", "--quiet", "-m", "b1"]);
        plain_git(&repo("project"), &["tag", "b1"]);
        let commits = Vec::new();
        Upstream {
            url: url("project"),
            dir,
            commits,
        }
    }

    #[test]
    fn named_submodules_are_checked_out_at_any_depth_in_any_order() {
        let upstream = deep_superproject();
        let orders: [&'static [&'static str]; 2] = [
            &["a", "a/b", "a/b/c", "a-extra", "version.txt"],
            &["version.txt", "a-extra", "a/b/c", "a/b", "a"],
        ];
        for submodules in orders {
            let root = tempfile::tempdir().unwrap();
            let sources = GitSources::new(root.path().to_owned());
            let recipe = Recipe {
                submodules,
                ..recipe(&upstream.url)
            };
            sources.fetch(&recipe).unwrap();
            let checked_out = sources
                .checkout(&recipe, "b1")
                .map(|checkout| checkout.dir)
                .map_err(|error| format!("{error:#}"));
            let dir = checked_out
                .as_ref()
                .map_or_else(|error| panic!("{submodules:?}: {error}"), PathBuf::clone);
            assert_eq!(
                version(&dir.join("a/b/c")).as_deref(),
                Some("c\n"),
                "{submodules:?}"
            );
            assert_eq!(
                version(&dir.join("a-extra")).as_deref(),
                Some("a-extra\n"),
                "{submodules:?}"
            );
            assert_eq!(
                version(&dir).as_deref(),
                Some("project\n"),
                "{submodules:?}"
            );
        }
    }

    #[test]
    fn a_submodule_that_cannot_be_fetched_fails_the_checkout() {
        let upstream = superproject();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = Recipe {
            submodules: &["ggml"],
            ..recipe(&upstream.url)
        };
        sources.fetch(&recipe).unwrap();
        fs::remove_dir_all(upstream.dir.path().join("ggml")).unwrap();
        let message = format!("{:#}", sources.checkout(&recipe, "b1").unwrap_err());
        assert!(
            message.starts_with("cannot check out the submodules ggml"),
            "{message}"
        );
    }

    /// Adds `added.txt`, then changes `version.txt` from `1` to `patched`:
    /// it applies to `b2` alone.
    const PATCH: Patch = Patch {
        name: "demo.patch",
        diff: "diff --git a/added.txt b/added.txt
new file mode 100644
--- /dev/null
+++ b/added.txt
@@ -0,0 +1 @@
+added
diff --git a/version.txt b/version.txt
--- a/version.txt
+++ b/version.txt
@@ -1 +1 @@
-1
+patched
",
    };

    #[test]
    fn a_patch_applies_whole_or_not_at_all() {
        let upstream = upstream();
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();

        let checkout = sources.checkout(&recipe, "b2").unwrap();
        sources.apply(&checkout, &PATCH).unwrap();
        assert_eq!(version(&checkout.dir).as_deref(), Some("patched\n"));
        assert!(checkout.dir.join("added.txt").is_file());
        // The patch file is gone; the next checkout undoes the patch.
        assert!(
            fs::read_dir(checkout.dir.join(".git"))
                .unwrap()
                .all(|entry| {
                    !entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .contains("demo.patch")
                })
        );
        sources.checkout(&recipe, "b2").unwrap();
        assert_eq!(version(&checkout.dir).as_deref(), Some("1\n"));
        assert!(!checkout.dir.join("added.txt").exists());

        // `version.txt` holds `2` at `b10`: nothing of the patch applies.
        let checkout = sources.checkout(&recipe, "b10").unwrap();
        let error = sources.apply(&checkout, &PATCH).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("patch failed: version.txt:1"), "{message}");
        assert!(error.downcast_ref::<PatchRefused>().is_some(), "{message}");
        assert_eq!(version(&checkout.dir).as_deref(), Some("2\n"));
        assert!(!checkout.dir.join("added.txt").exists());
    }

    #[test]
    fn a_tag_wins_over_a_branch_of_the_same_name() {
        let upstream = upstream();
        plain_git(upstream.dir.path(), &["branch", "b1", "feature"]);
        let root = tempfile::tempdir().unwrap();
        let sources = GitSources::new(root.path().to_owned());
        let recipe = recipe(&upstream.url);
        sources.fetch(&recipe).unwrap();
        let checkout = sources.checkout(&recipe, "b1").unwrap();
        assert_eq!(checkout.commit.as_str(), upstream.commits[0]);
    }
}
