//! Native builds: what was compiled, from which sources, with which
//! toolchain.
//!
//! A build is installed once and never modified; a project's builds sit side
//! by side, so going back to an older one needs no compilation.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use super::machine::{HostCompiler, Version};
use super::recipe::{Recipe, TagPattern};
use super::timestamp::Timestamp;

/// The CMake variable holding the CUDA architectures, `;`-separated.
pub const CUDA_ARCHITECTURES: &str = "CMAKE_CUDA_ARCHITECTURES";
/// The architecture native builds compile for: sm_120 with its
/// architecture-specific instructions (the `a`), such as the FP4 tensor
/// cores, as device code only (`-real`).
pub const CUDA_ARCHITECTURE: &str = "120a-real";
/// Hexadecimal digits of a commit kept in a build id.
const SHORT_COMMIT: usize = 10;
const MAX_ID_LEN: usize = 64;
/// The name of a project's link to its current build.
pub const CURRENT: &str = "current";

/// A build's name, unique within its project: the release tag it was built
/// from, such as `v1.0.0`, or the short commit otherwise. It names the
/// build's directory, so it is a safe file name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildId(String);

/// Why a text is not a build id.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "`{0}` is not a build id: 1 to 64 letters, digits, `.`, `_` or `-`, starting with a letter or a digit, and not `current`"
)]
pub struct InvalidBuildId(pub String);

impl BuildId {
    pub fn parse(text: &str) -> Result<Self, InvalidBuildId> {
        let valid = text.len() <= MAX_ID_LEN
            && text
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_alphanumeric())
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
            && text != CURRENT;
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(InvalidBuildId(text.to_owned()))
        }
    }

    /// The id of a build made from a commit that is not a release.
    pub fn from_commit(commit: &CommitId) -> Self {
        Self(commit.short().to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A full commit hash: 40 lower-case hexadecimal digits, or 64 in a
/// SHA-256 repository.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitId(String);

/// Why a text is not a full commit hash.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a full commit hash")]
pub struct InvalidCommitId(pub String);

impl CommitId {
    /// Accepts upper-case digits and stores them in lower case, as git
    /// prints them.
    pub fn parse(text: &str) -> Result<Self, InvalidCommitId> {
        let valid = matches!(text.len(), 40 | 64) && text.chars().all(|c| c.is_ascii_hexdigit());
        if valid {
            Ok(Self(text.to_ascii_lowercase()))
        } else {
            Err(InvalidCommitId(text.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The commit's first digits, as a build id made from it keeps them.
    pub fn short(&self) -> &str {
        &self.0[..SHORT_COMMIT]
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An installed build and how it was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    /// The recipe's project, such as `llama.cpp`.
    pub project: String,
    pub id: BuildId,
    /// The commit the sources were checked out at.
    pub commit: CommitId,
    /// The CUDA toolkit `nvcc` belonged to: the build loads that major
    /// version's CUDA libraries at run time.
    pub toolkit: Version,
    /// The compiler passed to CMake for C, C++ and as the CUDA host compiler.
    pub host_compiler: HostCompiler,
    /// Every `-D` option passed to CMake, by variable name.
    pub cmake_flags: BTreeMap<String, String>,
    /// The recipe's patches applied to the sources, by name, in order.
    pub patches: Vec<String>,
    pub built_at: Timestamp,
}

/// The order of builds from the newest, which `list` shows and `prune`
/// keeps; builds of the same time by id, the highest first.
pub fn newest_first(a: &Build, b: &Build) -> Ordering {
    b.built_at.cmp(&a.built_at).then_with(|| b.id.cmp(&a.id))
}

/// The builds `prune` removes from one project's: every build but the
/// `keep` newest and the current one.
pub fn to_prune<'a>(builds: &'a [Build], current: Option<&BuildId>, keep: usize) -> Vec<&'a Build> {
    let mut sorted: Vec<&Build> = builds.iter().collect();
    sorted.sort_by(|a, b| newest_first(a, b));
    sorted
        .into_iter()
        .skip(keep)
        .filter(|build| Some(&build.id) != current)
        .collect()
}

/// Where a project's current build stands against its newest release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// At the newest release, or past it, as after a release the project
    /// has withdrawn.
    Current,
    /// At an older release.
    Behind,
    /// No current build to run.
    NotBuilt,
    /// Built from a commit no release names, through `--ref`.
    FromCommit,
    /// At the newest release or past it, but not as its recipe builds it
    /// now: how.
    OffRecipe(String),
}

/// Where `build` stands against `newest`, a release of `tags`. `unlike` says
/// how `build` differs from its recipe, which matters at the newest release
/// or past it alone: any other standing is cured by a new build, which
/// follows the recipe.
pub fn standing(
    tags: TagPattern,
    build: Option<&Build>,
    newest: &str,
    unlike: Option<String>,
) -> Standing {
    let Some(build) = build else {
        return Standing::NotBuilt;
    };
    let Some(release) = tags.release(build.id.as_str()) else {
        return Standing::FromCommit;
    };
    if tags.release(newest).is_some_and(|newest| release < newest) {
        return Standing::Behind;
    }
    unlike.map_or(Standing::Current, Standing::OffRecipe)
}

/// The compilers a build runs, as the build gate settled them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolchain {
    nvcc: PathBuf,
    toolkit: Version,
    compiler: HostCompiler,
    c_compiler: PathBuf,
}

/// A toolchain path CMake cannot be given.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{} is not a UTF-8 path, which a CMake option needs", .0.display())]
pub struct NonUtf8Path(pub PathBuf);

impl Toolchain {
    /// `compiler` compiles C++ and is `nvcc`'s host compiler; `c_compiler`
    /// is the C compiler of the same GCC.
    pub fn new(
        nvcc: PathBuf,
        toolkit: Version,
        compiler: HostCompiler,
        c_compiler: PathBuf,
    ) -> Result<Self, NonUtf8Path> {
        for path in [&nvcc, &compiler.path, &c_compiler] {
            if path.to_str().is_none() {
                return Err(NonUtf8Path(path.clone()));
            }
        }
        Ok(Self {
            nvcc,
            toolkit,
            compiler,
            c_compiler,
        })
    }

    pub fn toolkit(&self) -> Version {
        self.toolkit
    }

    pub fn compiler(&self) -> &HostCompiler {
        &self.compiler
    }
}

/// The C compiler installed with a GNU C++ compiler: `gcc` beside `g++`,
/// `gcc-15` beside `g++-15`.
pub fn c_compiler_beside(cxx: &Path) -> Option<PathBuf> {
    let suffix = cxx.file_name()?.to_str()?.strip_prefix("g++")?;
    Some(cxx.with_file_name(format!("gcc{suffix}")))
}

/// Every `-D` option of the build `id`: the recipe's own, its release
/// options when `id` is a release, then the builder's, which take
/// precedence. Every compiler is named, so CMake finds neither a
/// compiler cache's wrapper nor another toolkit on the `PATH`; ggml still
/// launches the compilers through ccache when it is installed
/// (`GGML_CCACHE`, on by default), which only speeds up a rebuild.
pub fn cmake_flags(
    recipe: &Recipe,
    toolchain: &Toolchain,
    id: &BuildId,
) -> BTreeMap<String, String> {
    // `Toolchain::new` refused paths that are not UTF-8.
    let path = |path: &Path| path.to_string_lossy().into_owned();
    let release = recipe.tags.release(id.as_str()).is_some();
    let release_flags = recipe.release_flags.iter().filter(|_| release);
    let mut flags: BTreeMap<String, String> = recipe
        .cmake_flags
        .iter()
        .chain(release_flags)
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    let builder = [
        ("CMAKE_BUILD_TYPE", "Release".to_owned()),
        // llama.cpp and ggml inside the binary: nothing is loaded from the
        // build tree, which is removed after the build.
        ("BUILD_SHARED_LIBS", "OFF".to_owned()),
        ("GGML_CUDA", "ON".to_owned()),
        // CPU code tuned for this machine, the only one the build runs on.
        ("GGML_NATIVE", "ON".to_owned()),
        (CUDA_ARCHITECTURES, CUDA_ARCHITECTURE.to_owned()),
        ("CMAKE_CUDA_COMPILER", path(&toolchain.nvcc)),
        ("CMAKE_C_COMPILER", path(&toolchain.c_compiler)),
        ("CMAKE_CXX_COMPILER", path(&toolchain.compiler.path)),
        ("CMAKE_CUDA_HOST_COMPILER", path(&toolchain.compiler.path)),
    ];
    for (name, value) in builder {
        flags.insert(name.to_owned(), value);
    }
    flags
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    use super::*;
    use crate::domain::recipe::{LLAMA_CPP, STABLE_DIFFUSION_CPP, WHISPER_CPP};

    const COMMIT: &str = "79e2e74eb11022c1ba2e438df7f0ca2d4c10f8b6";

    #[test]
    fn release_tags_and_short_commits_are_build_ids() {
        for id in [
            "b11538",
            "79e2e74eb1",
            "v1.8.2",
            "master-480-ce3e7b1",
            "a_b",
        ] {
            assert_eq!(BuildId::parse(id).unwrap().as_str(), id);
        }
    }

    #[test]
    fn unsafe_names_are_not_build_ids() {
        let too_long = "b".repeat(MAX_ID_LEN + 1);
        for id in [
            "",
            ".",
            "..",
            ".hidden",
            "-rf",
            "a/b",
            "a b",
            "été",
            "current",
            too_long.as_str(),
        ] {
            assert_eq!(
                BuildId::parse(id),
                Err(InvalidBuildId(id.to_owned())),
                "{id}"
            );
        }
        assert!(BuildId::parse(&"b".repeat(MAX_ID_LEN)).is_ok());
    }

    #[test]
    fn a_commit_build_id_keeps_ten_digits() {
        let commit = CommitId::parse(COMMIT).unwrap();
        assert_eq!(BuildId::from_commit(&commit).as_str(), "79e2e74eb1");
    }

    #[test]
    fn commit_ids_are_full_hexadecimal_hashes() {
        assert_eq!(
            CommitId::parse(&COMMIT.to_ascii_uppercase())
                .unwrap()
                .as_str(),
            COMMIT
        );
        assert!(CommitId::parse(&"a".repeat(64)).is_ok());
        for text in [
            "",
            "79e2e74",
            &COMMIT[..39],
            &format!("{COMMIT}0"),
            &COMMIT.replace('e', "g"),
        ] {
            assert!(CommitId::parse(text).is_err(), "{text}");
        }
    }

    fn gxx(path: &str, version: Version) -> HostCompiler {
        HostCompiler {
            command: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            path: PathBuf::from(path),
            version,
        }
    }

    fn toolchain(cxx: &str) -> Toolchain {
        Toolchain::new(
            PathBuf::from("/usr/local/cuda-13.4/bin/nvcc"),
            Version::new(13, 4, 92),
            gxx(cxx, Version::new(15, 3, 1)),
            c_compiler_beside(Path::new(cxx)).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn the_c_compiler_sits_beside_the_cxx_compiler() {
        assert_eq!(
            c_compiler_beside(Path::new("/usr/bin/g++")),
            Some(PathBuf::from("/usr/bin/gcc"))
        );
        assert_eq!(
            c_compiler_beside(Path::new("/usr/bin/g++-15")),
            Some(PathBuf::from("/usr/bin/gcc-15"))
        );
        assert_eq!(c_compiler_beside(Path::new("/usr/bin/clang++")), None);
    }

    #[test]
    fn a_toolchain_path_must_be_utf8() {
        let odd = PathBuf::from(OsStr::from_bytes(b"/opt/cuda-\xff/bin/nvcc"));
        let error = Toolchain::new(
            odd.clone(),
            Version::new(13, 4, 92),
            gxx("/usr/bin/g++", Version::new(16, 2, 1)),
            PathBuf::from("/usr/bin/gcc"),
        )
        .unwrap_err();
        assert_eq!(error, NonUtf8Path(odd));
    }

    #[test]
    fn cmake_flags_name_every_compiler_and_the_blackwell_architecture() {
        let flags = cmake_flags(&LLAMA_CPP, &toolchain("/usr/bin/g++-15"), &commit_id());
        let expected = [
            ("BUILD_SHARED_LIBS", "OFF"),
            ("CMAKE_BUILD_TYPE", "Release"),
            ("CMAKE_CUDA_ARCHITECTURES", "120a-real"),
            ("CMAKE_CUDA_COMPILER", "/usr/local/cuda-13.4/bin/nvcc"),
            ("CMAKE_CUDA_HOST_COMPILER", "/usr/bin/g++-15"),
            ("CMAKE_CXX_COMPILER", "/usr/bin/g++-15"),
            ("CMAKE_C_COMPILER", "/usr/bin/gcc-15"),
            ("GGML_CUDA", "ON"),
            ("GGML_NATIVE", "ON"),
            ("LLAMA_BUILD_APP", "OFF"),
            ("LLAMA_BUILD_EXAMPLES", "OFF"),
            ("LLAMA_BUILD_SERVER", "ON"),
            ("LLAMA_BUILD_TESTS", "OFF"),
            ("LLAMA_BUILD_TOOLS", "ON"),
            ("LLAMA_USE_PREBUILT_UI", "OFF"),
        ];
        let flags: Vec<(&str, &str)> = flags
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        assert_eq!(flags, expected);
    }

    #[test]
    fn the_builder_options_win_over_the_recipe() {
        let recipe = Recipe {
            cmake_flags: &[("GGML_CUDA", "OFF"), ("WHISPER_BUILD_TESTS", "OFF")],
            ..LLAMA_CPP
        };
        let flags = cmake_flags(&recipe, &toolchain("/usr/bin/g++"), &commit_id());
        assert_eq!(flags["GGML_CUDA"], "ON");
        assert_eq!(flags["WHISPER_BUILD_TESTS"], "OFF");
    }

    #[test]
    fn the_builder_overrides_no_option_a_recipe_sets() {
        // A build is compared with its recipe's options: one the builder
        // overrode would leave every build unlike its recipe.
        for recipe in crate::domain::recipe::RECIPES {
            let flags = cmake_flags(recipe, &toolchain("/usr/bin/g++"), &commit_id());
            for (name, value) in recipe.cmake_flags {
                assert_eq!(
                    flags.get(*name).map(String::as_str),
                    Some(*value),
                    "{}: {name}",
                    recipe.project
                );
            }
        }
    }

    /// The id of a build made from a commit no release tags.
    fn commit_id() -> BuildId {
        BuildId::from_commit(&CommitId::parse(COMMIT).unwrap())
    }

    #[test]
    fn a_release_alone_takes_its_recipes_release_options() {
        let release = BuildId::parse("v1.0.0").unwrap();
        let flags = cmake_flags(&LLAMA_CPP, &toolchain("/usr/bin/g++"), &release);
        assert_eq!(flags["LLAMA_BUILD_IS_DEV"], "OFF");
        assert_eq!(flags["LLAMA_USE_PREBUILT_UI"], "OFF");
        for id in [commit_id(), BuildId::parse("b12359").unwrap()] {
            let flags = cmake_flags(&LLAMA_CPP, &toolchain("/usr/bin/g++"), &id);
            assert_eq!(flags.get("LLAMA_BUILD_IS_DEV"), None, "{id}");
        }
        // The builder's options win over them too.
        let recipe = Recipe {
            release_flags: &[("GGML_NATIVE", "OFF")],
            ..LLAMA_CPP
        };
        let flags = cmake_flags(&recipe, &toolchain("/usr/bin/g++"), &release);
        assert_eq!(flags["GGML_NATIVE"], "ON");
    }

    fn built(id: &str, at: i64) -> Build {
        Build {
            project: "llama.cpp".into(),
            id: BuildId::parse(id).unwrap(),
            commit: CommitId::parse(COMMIT).unwrap(),
            toolkit: Version::new(13, 4, 92),
            host_compiler: gxx("/usr/bin/g++", Version::new(16, 2, 1)),
            cmake_flags: BTreeMap::new(),
            patches: vec![],
            built_at: Timestamp::from_unix(at).unwrap(),
        }
    }

    fn ids(builds: Vec<&Build>) -> Vec<&str> {
        builds.iter().map(|build| build.id.as_str()).collect()
    }

    fn standing_of(tags: TagPattern, id: &str, newest: &str) -> Standing {
        standing(tags, Some(&built(id, 10)), newest, None)
    }

    #[test]
    fn a_build_at_its_newest_release_is_current_in_each_form() {
        for (tags, release) in [
            (LLAMA_CPP.tags, "v1.0.0"),
            (WHISPER_CPP.tags, "v1.9.5"),
            (STABLE_DIFFUSION_CPP.tags, "master-1043-0a1b2c3"),
        ] {
            assert_eq!(
                standing_of(tags, release, release),
                Standing::Current,
                "{release}"
            );
        }
    }

    #[test]
    fn an_older_release_is_behind_by_its_numbers_in_each_form() {
        for (tags, id, newest) in [
            (LLAMA_CPP.tags, "v0.9.0", "v1.0.0"),
            (WHISPER_CPP.tags, "v1.9.5", "v1.10.0"),
            (WHISPER_CPP.tags, "v0.9.3", "v0.9.4"),
            (
                STABLE_DIFFUSION_CPP.tags,
                "master-999-77b72a2",
                "master-1043-0a1b2c3",
            ),
        ] {
            assert_eq!(standing_of(tags, id, newest), Standing::Behind, "{id}");
        }
    }

    #[test]
    fn a_build_past_the_newest_release_is_never_sent_back() {
        // The project withdrew the release this build was made from.
        assert_eq!(
            standing_of(LLAMA_CPP.tags, "v1.0.1", "v1.0.0"),
            Standing::Current
        );
        assert_eq!(
            standing_of(WHISPER_CPP.tags, "v1.10.0", "v1.9.5"),
            Standing::Current
        );
    }

    #[test]
    fn a_build_made_from_a_commit_is_not_a_release() {
        assert_eq!(
            standing_of(LLAMA_CPP.tags, "79e2e74eb1", "v1.0.0"),
            Standing::FromCommit
        );
        // llama.cpp's nightly tags, which ai-ctl built until it took its
        // releases `vX.Y.Z`.
        assert_eq!(
            standing_of(LLAMA_CPP.tags, "b12359", "v1.0.0"),
            Standing::FromCommit
        );
        // An id that is no release of the project, whatever made it.
        assert_eq!(
            standing_of(WHISPER_CPP.tags, "b12177", "v1.9.5"),
            Standing::FromCommit
        );
    }

    #[test]
    fn a_project_with_no_build_is_not_built() {
        assert_eq!(
            standing(LLAMA_CPP.tags, None, "v1.0.0", None),
            Standing::NotBuilt
        );
    }

    #[test]
    fn only_the_newest_release_can_be_off_its_recipe() {
        let unlike = || Some("v1.0.0 was built without a patch".to_owned());
        let at = |id: &str| standing(LLAMA_CPP.tags, Some(&built(id, 10)), "v1.0.0", unlike());
        assert_eq!(
            at("v1.0.0"),
            Standing::OffRecipe("v1.0.0 was built without a patch".to_owned())
        );
        assert_eq!(
            at("v1.0.1"),
            Standing::OffRecipe("v1.0.0 was built without a patch".to_owned())
        );
        // A new build follows the recipe: being behind says all there is.
        assert_eq!(at("v0.9.0"), Standing::Behind);
        assert_eq!(at("79e2e74eb1"), Standing::FromCommit);
    }

    #[test]
    fn prune_keeps_the_newest_builds() {
        let builds = [
            built("b1", 10),
            built("b3", 30),
            built("b2", 20),
            built("b4", 40),
        ];
        assert_eq!(ids(to_prune(&builds, None, 2)), ["b2", "b1"]);
        assert_eq!(ids(to_prune(&builds, None, 0)), ["b4", "b3", "b2", "b1"]);
        assert!(to_prune(&builds, None, 4).is_empty());
        assert!(to_prune(&builds, None, 9).is_empty());
        assert!(to_prune(&[], None, 0).is_empty());
    }

    #[test]
    fn prune_never_removes_the_current_build() {
        let builds = [built("b1", 10), built("b2", 20), built("b3", 30)];
        let current = BuildId::parse("b1").unwrap();
        assert_eq!(ids(to_prune(&builds, Some(&current), 1)), ["b2"]);
        assert!(to_prune(&builds[..1], Some(&current), 0).is_empty());
    }

    #[test]
    fn builds_of_the_same_second_prune_in_id_order() {
        let builds = [built("aaaaaaaaaa", 10), built("bbbbbbbbbb", 10)];
        assert_eq!(ids(to_prune(&builds, None, 1)), ["aaaaaaaaaa"]);
    }
}
