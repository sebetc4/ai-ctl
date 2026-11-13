//! Build recipes: where a ggml project's sources live, how its releases are
//! tagged, and which CMake target makes its server.
//!
//! The builder is the same for every project; everything specific to one
//! project is in its recipe, its patches in `patches/<project>/`.

use std::path::Path;

/// How to fetch and compile one project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recipe {
    /// The project's name, as the inventory's `source = { build = "…" }`
    /// gives it.
    pub project: &'static str,
    /// The git repository the sources are cloned from.
    pub repository: &'static str,
    pub tags: TagPattern,
    /// The submodules the build needs, by path, a submodule's own by its
    /// whole path after it: the builder initialises these and no other,
    /// those a release holds.
    pub submodules: &'static [&'static str],
    /// The changes ai-ctl makes to the sources before it compiles them, in
    /// the order they apply.
    pub patches: &'static [Patch],
    /// The project's own CMake options; the builder adds the CUDA and
    /// toolchain ones.
    pub cmake_flags: &'static [(&'static str, &'static str)],
    /// The CMake target to build, and nothing else.
    pub target: &'static str,
    /// Where the target's binary lands, relative to the CMake build tree.
    pub artifact: &'static str,
    /// How the new binary shows that it runs and is the project's server.
    pub smoke_test: SmokeTest,
    /// How the binary lists the devices it sees, when it can.
    pub device_listing: Option<DeviceListing>,
}

impl Recipe {
    /// The installed binary's file name, such as `llama-server`.
    pub fn binary(&self) -> &'static str {
        Path::new(self.artifact)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(self.artifact)
    }
}

/// A change ai-ctl makes to a project's sources, such as the check of the
/// key a server lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    /// The file name the build records it under, such as `api-key.patch`.
    pub name: &'static str,
    /// A unified diff, as `git diff` prints it.
    pub diff: &'static str,
}

/// A patch the checked-out tree refused, with what the tool said: the one
/// failure another release may not meet.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct PatchRefused(pub String);

/// How a project names its release tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagPattern {
    /// A prefix and a release number, such as llama.cpp's `b6512`.
    Numbered { prefix: &'static str },
    /// A prefix, a release number and the short hash of its commit, such as
    /// stable-diffusion.cpp's `master-975-95b3090`.
    NumberedWithCommit { prefix: &'static str },
    /// A prefix and a version `X.Y.Z`, such as whisper.cpp's `v1.9.5`;
    /// nothing may follow it, so `v1.7.4-pre-1` is not a release.
    Version { prefix: &'static str },
}

impl TagPattern {
    /// The numbers the tag's release is ordered by, or `None` when the tag
    /// is not a release.
    pub fn release(&self, tag: &str) -> Option<Vec<u64>> {
        match self {
            TagPattern::Numbered { prefix } => Some(vec![number(tag.strip_prefix(prefix)?)?]),
            TagPattern::NumberedWithCommit { prefix } => {
                let (release, hash) = tag.strip_prefix(prefix)?.split_once('-')?;
                let release = number(release)?;
                let hash = !hash.is_empty()
                    && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
                hash.then(|| vec![release])
            }
            TagPattern::Version { prefix } => {
                let numbers = tag.strip_prefix(prefix)?.split('.').map(number);
                let numbers: Vec<u64> = numbers.collect::<Option<_>>()?;
                (numbers.len() == 3).then_some(numbers)
            }
        }
    }

    /// The newest release among `tags`; other tags are ignored.
    pub fn newest<'a>(&self, tags: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
        tags.into_iter()
            .filter_map(|tag| self.release(tag).map(|numbers| (numbers, tag)))
            .max_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, tag)| tag)
    }

    /// The glob `git tag --list` narrows the tags with. It lets through
    /// tags that are not releases, such as `b10964-b10964-b29c606`:
    /// [`TagPattern::release`] has the last word.
    pub fn glob(&self) -> String {
        match self {
            TagPattern::Numbered { prefix }
            | TagPattern::NumberedWithCommit { prefix }
            | TagPattern::Version { prefix } => format!("{prefix}[0-9]*"),
        }
    }
}

/// A release number: decimal digits only, so that `+1` is not one.
fn number(text: &str) -> Option<u64> {
    let digits = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    if digits { text.parse().ok() } else { None }
}

/// Arguments the new binary runs with, and a text its output must hold:
/// its exit status alone proves little, since `whisper-server` exits 0 on
/// an argument it does not know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmokeTest {
    pub args: &'static [&'static str],
    pub prints: &'static str,
}

/// Arguments that make a binary print the devices it sees, one per line,
/// such as `  CUDA0: NVIDIA GeForce RTX 5090 Laptop GPU (24463 MiB, …)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceListing {
    pub args: &'static [&'static str],
}

/// The CUDA devices in a device listing: the lines naming `CUDA<n>`, then
/// `:` as llama-server prints them or a tab as sd-server does, or naming
/// `CUDA:<n>` then a space, as audiocpp_server does.
pub fn cuda_devices(listing: &str) -> Vec<&str> {
    listing
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.strip_prefix("CUDA").is_some_and(|device| {
                let (index, separators) = match device.strip_prefix(':') {
                    Some(index) => (index, &[' '][..]),
                    None => (device, &[':', '\t'][..]),
                };
                let digits = index.bytes().take_while(u8::is_ascii_digit).count();
                digits > 0 && index[digits..].starts_with(separators)
            })
        })
        .collect()
}

/// llama.cpp: releases are tagged `b<number>` several times a day; the
/// server is the `llama-server` target, in `tools/server`, built without
/// its web UI.
pub const LLAMA_CPP: Recipe = Recipe {
    project: "llama.cpp",
    repository: "https://github.com/ggml-org/llama.cpp.git",
    tags: TagPattern::Numbered { prefix: "b" },
    submodules: &[],
    patches: &[],
    cmake_flags: &[
        // `tools/` holds the server; only the target below is compiled.
        ("LLAMA_BUILD_TOOLS", "ON"),
        ("LLAMA_BUILD_SERVER", "ON"),
        ("LLAMA_BUILD_TESTS", "OFF"),
        ("LLAMA_BUILD_EXAMPLES", "OFF"),
        // The unified `llama` binary, which is not needed.
        ("LLAMA_BUILD_APP", "OFF"),
        // No web UI: the build would download it from a Hugging Face
        // bucket, falling back to its mutable `latest`, outside the commit,
        // and compile its file names into the server's code.
        ("LLAMA_USE_PREBUILT_UI", "OFF"),
    ],
    target: "llama-server",
    artifact: "bin/llama-server",
    smoke_test: SmokeTest {
        args: &["--version"],
        prints: "version: ",
    },
    device_listing: Some(DeviceListing {
        args: &["--list-devices"],
    }),
};

/// whisper.cpp: releases are tagged `vX.Y.Z`; the server is the
/// `whisper-server` target, in `examples/server`, which ggml's own examples
/// option adds. It has neither `--version` nor a device listing: its help
/// shows it runs. Its patches make it require the gateway's key and bind as
/// llama-server does, refusing a port another server listens on.
pub const WHISPER_CPP: Recipe = Recipe {
    project: "whisper.cpp",
    repository: "https://github.com/ggml-org/whisper.cpp.git",
    tags: TagPattern::Version { prefix: "v" },
    submodules: &[],
    patches: &[
        Patch {
            name: "api-key.patch",
            diff: include_str!("patches/whisper.cpp/api-key.patch"),
        },
        Patch {
            name: "reuse-address.patch",
            diff: include_str!("patches/whisper.cpp/reuse-address.patch"),
        },
    ],
    cmake_flags: &[
        ("WHISPER_BUILD_TESTS", "OFF"),
        ("WHISPER_BUILD_EXAMPLES", "ON"),
    ],
    target: "whisper-server",
    artifact: "bin/whisper-server",
    smoke_test: SmokeTest {
        args: &["--help"],
        prints: "--inference-path",
    },
    device_listing: None,
};

/// stable-diffusion.cpp: releases are tagged `master-<N>-<hash>` several
/// times a day; the server is the `sd-server` target, in `examples/server`.
/// Of its submodules, only its fork of ggml is needed, with the CUTLASS its
/// CUDA kernels are built with: the fork's own from `master-1024`, then the
/// project's, in `thirdparty/`, a release holding one or the other. WebP and
/// WebM are off, and the web frontend, which `pnpm install` would fetch at
/// build time, is not built. Its patches make it require the gateway's key, end a
/// failed `listen` with an error, where it exits 0 without a word, and bind
/// as llama-server does, refusing a port another server listens on.
pub const STABLE_DIFFUSION_CPP: Recipe = Recipe {
    project: "stable-diffusion.cpp",
    repository: "https://github.com/leejet/stable-diffusion.cpp.git",
    tags: TagPattern::NumberedWithCommit { prefix: "master-" },
    submodules: &[
        "ggml",
        "ggml/src/ggml-cuda/vendors/cutlass",
        "thirdparty/cutlass",
    ],
    patches: &[
        Patch {
            name: "api-key.patch",
            diff: include_str!("patches/stable-diffusion.cpp/api-key.patch"),
        },
        Patch {
            name: "listen-failure.patch",
            diff: include_str!("patches/stable-diffusion.cpp/listen-failure.patch"),
        },
        Patch {
            name: "reuse-address.patch",
            diff: include_str!("patches/stable-diffusion.cpp/reuse-address.patch"),
        },
    ],
    cmake_flags: &[
        ("SD_CUDA", "ON"),
        ("SD_BUILD_EXAMPLES", "ON"),
        ("SD_SERVER_BUILD_FRONTEND", "OFF"),
        ("SD_WEBP", "OFF"),
        ("SD_WEBM", "OFF"),
    ],
    target: "sd-server",
    artifact: "bin/sd-server",
    smoke_test: SmokeTest {
        args: &["--version"],
        prints: "stable-diffusion.cpp version",
    },
    device_listing: Some(DeviceListing {
        args: &["--list-devices"],
    }),
};

/// audio.cpp: releases are tagged `vX.Y.Z`; the server is the
/// `audiocpp_server` target. ggml is vendored, and its one submodule, of
/// optional modules, has an SSH URL that needs a key to fetch: it is left
/// out. A deployment build compiles the model specs of its commit into the
/// binary, without which a model from a publisher's safetensors cannot
/// start. Its patch makes it require the gateway's key.
pub const AUDIO_CPP: Recipe = Recipe {
    project: "audio.cpp",
    repository: "https://github.com/0xShug0/audio.cpp.git",
    tags: TagPattern::Version { prefix: "v" },
    submodules: &[],
    patches: &[Patch {
        name: "api-key.patch",
        diff: include_str!("patches/audio.cpp/api-key.patch"),
    }],
    cmake_flags: &[
        ("ENGINE_ENABLE_CUDA", "ON"),
        ("ENGINE_ENABLE_NATIVE_CPU", "ON"),
        ("AUDIOCPP_DEPLOYMENT_BUILD", "ON"),
    ],
    target: "audiocpp_server",
    artifact: "bin/audiocpp_server",
    smoke_test: SmokeTest {
        args: &["--version"],
        prints: "audio.cpp ",
    },
    device_listing: Some(DeviceListing {
        args: &["--list-devices"],
    }),
};

/// Every recipe ai-ctl ships.
pub const RECIPES: &[Recipe] = &[LLAMA_CPP, WHISPER_CPP, STABLE_DIFFUSION_CPP, AUDIO_CPP];

/// The recipe of `project`.
pub fn recipe(project: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| recipe.project == project)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LLAMA_TAGS: TagPattern = TagPattern::Numbered { prefix: "b" };
    const SD_CPP_TAGS: TagPattern = TagPattern::NumberedWithCommit { prefix: "master-" };
    const VERSION_TAGS: TagPattern = TagPattern::Version { prefix: "v" };

    #[test]
    fn llama_cpp_is_the_shipped_recipe() {
        let recipe = recipe("llama.cpp").unwrap();
        assert_eq!(recipe.target, "llama-server");
        assert_eq!(recipe.binary(), "llama-server");
        assert!(recipe.repository.starts_with("https://"));
        assert_eq!(super::recipe("nope.cpp"), None);
        // Its web UI would be downloaded at build time from a mutable
        // bucket, outside the commit, and compiled into the server.
        assert!(
            recipe
                .cmake_flags
                .contains(&("LLAMA_USE_PREBUILT_UI", "OFF"))
        );
    }

    #[test]
    fn whisper_cpp_is_built_as_phase_0_built_it_by_hand() {
        let recipe = recipe("whisper.cpp").expect("whisper.cpp is shipped");
        assert_eq!(recipe.binary(), "whisper-server");
        let tags = include_str!("../../tests/fixtures/whisper.cpp-git-tag-list.txt");
        assert_eq!(recipe.tags.newest(tags.lines()), Some("v1.9.5"));
        // No `--version` and no device listing: its help shows it runs.
        let help = include_str!("../../tests/fixtures/whisper-server-v1.9.5-help.txt");
        assert!(help.contains(recipe.smoke_test.prints));
        assert_eq!(recipe.device_listing, None);
        let patches: Vec<&str> = recipe.patches.iter().map(|patch| patch.name).collect();
        assert_eq!(patches, ["api-key.patch", "reuse-address.patch"]);
    }

    #[test]
    fn stable_diffusion_cpp_is_built_as_phase_0_built_it_by_hand() {
        let recipe = recipe("stable-diffusion.cpp").expect("stable-diffusion.cpp is shipped");
        assert_eq!(recipe.binary(), "sd-server");
        let tags = include_str!("../../tests/fixtures/stable-diffusion.cpp-git-tag-list.txt");
        assert_eq!(recipe.tags.newest(tags.lines()), Some("master-975-95b3090"));
        // Its fork of ggml alone, with the CUTLASS ggml builds with from
        // `master-1024` and the one sd.cpp holds itself by `master-1071`:
        // the other submodules would add WebP, WebM and a frontend that
        // `pnpm install` fetches at build time.
        assert_eq!(
            recipe.submodules,
            [
                "ggml",
                "ggml/src/ggml-cuda/vendors/cutlass",
                "thirdparty/cutlass"
            ]
        );
        let version = include_str!("../../tests/fixtures/sd-server-master-975-version.txt");
        assert!(version.contains(recipe.smoke_test.prints));
        let listing = include_str!("../../tests/fixtures/sd-server-master-975-list-devices.txt");
        assert_eq!(
            recipe.device_listing.map(|listing| listing.args),
            Some(&["--list-devices"][..])
        );
        assert_eq!(cuda_devices(listing).len(), 1);
        let patches: Vec<&str> = recipe.patches.iter().map(|patch| patch.name).collect();
        assert_eq!(
            patches,
            [
                "api-key.patch",
                "listen-failure.patch",
                "reuse-address.patch"
            ]
        );
    }

    #[test]
    fn audio_cpp_is_built_as_phase_0_built_it_by_hand() {
        let recipe = recipe("audio.cpp").expect("audio.cpp is shipped");
        assert_eq!(recipe.binary(), "audiocpp_server");
        let tags = include_str!("../../tests/fixtures/audio.cpp-git-tag-list.txt");
        assert_eq!(recipe.tags.newest(tags.lines()), Some("v0.9.2"));
        // Its one submodule has an SSH URL, which needs a key to fetch.
        assert!(recipe.submodules.is_empty());
        let version = include_str!("../../tests/fixtures/audiocpp_server-v0.9.2-version.txt");
        assert!(version.contains(recipe.smoke_test.prints));
        let listing = include_str!("../../tests/fixtures/audiocpp_server-v0.9.2-list-devices.txt");
        assert_eq!(
            recipe.device_listing.map(|listing| listing.args),
            Some(&["--list-devices"][..])
        );
        assert_eq!(cuda_devices(listing).len(), 1);
        let patches: Vec<&str> = recipe.patches.iter().map(|patch| patch.name).collect();
        assert_eq!(patches, ["api-key.patch"]);
    }

    #[test]
    fn each_new_recipe_keeps_the_repository_and_options_it_is_built_with() {
        let whisper = recipe("whisper.cpp").unwrap();
        assert_eq!(
            whisper.repository,
            "https://github.com/ggml-org/whisper.cpp.git"
        );
        assert_eq!(
            whisper.cmake_flags,
            [
                ("WHISPER_BUILD_TESTS", "OFF"),
                ("WHISPER_BUILD_EXAMPLES", "ON")
            ]
        );
        let sd = recipe("stable-diffusion.cpp").unwrap();
        assert_eq!(
            sd.repository,
            "https://github.com/leejet/stable-diffusion.cpp.git"
        );
        // Without the frontend off, the build runs `pnpm install`.
        assert_eq!(
            sd.cmake_flags,
            [
                ("SD_CUDA", "ON"),
                ("SD_BUILD_EXAMPLES", "ON"),
                ("SD_SERVER_BUILD_FRONTEND", "OFF"),
                ("SD_WEBP", "OFF"),
                ("SD_WEBM", "OFF"),
            ]
        );
        let audio = recipe("audio.cpp").unwrap();
        assert_eq!(audio.repository, "https://github.com/0xShug0/audio.cpp.git");
        // Without a deployment build, a model from safetensors cannot start.
        assert_eq!(
            audio.cmake_flags,
            [
                ("ENGINE_ENABLE_CUDA", "ON"),
                ("ENGINE_ENABLE_NATIVE_CPU", "ON"),
                ("AUDIOCPP_DEPLOYMENT_BUILD", "ON"),
            ]
        );
    }

    #[test]
    fn shipped_patches_are_diffs_under_plain_names() {
        for recipe in RECIPES {
            for (i, patch) in recipe.patches.iter().enumerate() {
                let stem = patch.name.strip_suffix(".patch").unwrap_or("");
                assert!(
                    !stem.is_empty()
                        && stem
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                    "{}: {}",
                    recipe.project,
                    patch.name
                );
                assert!(
                    recipe.patches[i + 1..]
                        .iter()
                        .all(|other| other.name != patch.name),
                    "{}: {} twice",
                    recipe.project,
                    patch.name
                );
                assert!(patch.diff.starts_with("diff --git a/"), "{}", patch.name);
                assert!(patch.diff.ends_with('\n'), "{}", patch.name);
            }
        }
    }

    #[test]
    fn recipe_projects_are_unique() {
        for (i, recipe) in RECIPES.iter().enumerate() {
            assert!(
                RECIPES[i + 1..]
                    .iter()
                    .all(|other| other.project != recipe.project),
                "{} is shipped twice",
                recipe.project
            );
        }
    }

    #[test]
    fn recipes_leave_toolchain_and_cuda_options_to_the_builder() {
        for recipe in RECIPES {
            for (name, _) in recipe.cmake_flags {
                assert!(
                    !name.starts_with("CMAKE_") && !name.starts_with("GGML_CUDA"),
                    "{}: {name}",
                    recipe.project
                );
            }
        }
    }

    #[test]
    fn llama_cpp_release_tags_are_numbered() {
        assert_eq!(LLAMA_TAGS.release("b11538"), Some(vec![11538]));
        assert_eq!(LLAMA_TAGS.release("b1"), Some(vec![1]));
        for tag in [
            "b",
            "b10964-b10964-b29c606",
            "backup/allozaur-ui",
            "11538",
            "B11538",
            "b+1",
        ] {
            assert_eq!(LLAMA_TAGS.release(tag), None, "{tag}");
        }
    }

    #[test]
    fn the_newest_release_is_compared_numerically() {
        let tags = [
            "b9999",
            "b10964-b10964-b29c606",
            "b11538",
            "b10000",
            "gguf-v0.17.1",
        ];
        assert_eq!(LLAMA_TAGS.newest(tags), Some("b11538"));
        assert_eq!(LLAMA_TAGS.newest(["b9", "b10"]), Some("b10"));
        assert_eq!(LLAMA_TAGS.newest(["master"]), None);
        assert_eq!(LLAMA_TAGS.newest([]), None);
    }

    #[test]
    fn stable_diffusion_cpp_releases_carry_their_commit() {
        assert_eq!(SD_CPP_TAGS.release("master-975-95b3090"), Some(vec![975]));
        assert_eq!(SD_CPP_TAGS.release("master-1-a"), Some(vec![1]));
        for tag in [
            // The older form, whose hash can be all digits.
            "master-0739361",
            "master-004dfbe",
            "workflow-302-fad6178",
            "master-975",
            "master-975-",
            "master--95b3090",
            "master-+975-95b3090",
            "master-975-95B3090",
            "master-975-95b3090-x",
            "master-975-95g3090",
        ] {
            assert_eq!(SD_CPP_TAGS.release(tag), None, "{tag}");
        }
    }

    #[test]
    fn version_releases_are_three_numbers_and_nothing_after() {
        assert_eq!(VERSION_TAGS.release("v1.9.5"), Some(vec![1, 9, 5]));
        assert_eq!(VERSION_TAGS.release("v0.10.20"), Some(vec![0, 10, 20]));
        for tag in [
            "v1.7.4-pre-1",
            "v1.7.2-pre",
            "v0.9.2-drama3-base-small",
            "v0.1.0-windows-prebuilt",
            "release-0.4.2",
            "1.0.3",
            "b2250",
            "V1.9.5",
            "v1.9",
            "v1.9.5.1",
            "v1..5",
            "v1.9.",
            "v.1.9",
            "v1.+9.5",
        ] {
            assert_eq!(VERSION_TAGS.release(tag), None, "{tag}");
        }
    }

    #[test]
    fn versions_are_compared_number_by_number() {
        assert_eq!(
            VERSION_TAGS.newest(["v1.9.5", "v1.10.0", "v1.9.10"]),
            Some("v1.10.0")
        );
        assert_eq!(VERSION_TAGS.newest(["v1.9.10", "v1.9.5"]), Some("v1.9.10"));
        assert_eq!(VERSION_TAGS.newest(["v0.9.2", "v1.0.0"]), Some("v1.0.0"));
    }

    #[test]
    fn the_newest_release_of_each_captured_tag_list() {
        // `git tag --list` of each clone, captured on 2026-10-20.
        let lists = [
            (
                VERSION_TAGS,
                include_str!("../../tests/fixtures/whisper.cpp-git-tag-list.txt"),
                40,
                "v1.9.5",
            ),
            (
                VERSION_TAGS,
                include_str!("../../tests/fixtures/audio.cpp-git-tag-list.txt"),
                11,
                "v0.9.2",
            ),
            (
                SD_CPP_TAGS,
                include_str!("../../tests/fixtures/stable-diffusion.cpp-git-tag-list.txt"),
                539,
                "master-975-95b3090",
            ),
        ];
        for (pattern, list, releases, newest) in lists {
            let found = list.lines().filter(|tag| pattern.release(tag).is_some());
            assert_eq!(found.count(), releases, "{newest}");
            assert_eq!(pattern.newest(list.lines()), Some(newest));
        }
    }

    #[test]
    fn ls_remote_narrows_to_tags_starting_like_releases() {
        assert_eq!(LLAMA_TAGS.glob(), "b[0-9]*");
        assert_eq!(SD_CPP_TAGS.glob(), "master-[0-9]*");
        assert_eq!(VERSION_TAGS.glob(), "v[0-9]*");
    }

    #[test]
    fn cuda_devices_are_read_from_the_listing() {
        // `llama-server --list-devices` of b11538 on this machine.
        let listing = include_str!("../../tests/fixtures/llama-server-list-devices.txt");
        assert_eq!(
            cuda_devices(listing),
            ["CUDA0: NVIDIA GeForce RTX 5090 Laptop GPU (24075 MiB, 23798 MiB free)"]
        );
        assert!(cuda_devices("Available devices:\n").is_empty());
        assert!(cuda_devices("CUDA: no device\nCUDAx: y\n  Vulkan0: other\n").is_empty());
    }

    #[test]
    fn cuda_devices_are_read_from_each_servers_listing() {
        // `--list-devices` of phase 0's builds, both streams.
        let sd_server = include_str!("../../tests/fixtures/sd-server-master-975-list-devices.txt");
        assert_eq!(
            cuda_devices(sd_server),
            ["CUDA0\tNVIDIA GeForce RTX 5090 Laptop GPU"]
        );
        let audiocpp_server =
            include_str!("../../tests/fixtures/audiocpp_server-v0.9.2-list-devices.txt");
        assert_eq!(
            cuda_devices(audiocpp_server),
            ["CUDA:0 \"NVIDIA GeForce RTX 5090 Laptop GPU\" [GPU]"]
        );
        assert_eq!(
            cuda_devices("CUDA12\tx\nCUDA:12 x\n"),
            ["CUDA12\tx", "CUDA:12 x"]
        );
        for line in [
            "CUDA0",
            "CUDA0 x",
            "CUDA:0",
            "CUDA:0\tx",
            "CUDA:0:x",
            "CUDA: 0 x",
            "CUDA:x y",
            "CUDA\tx",
            "ggml_cuda_init: found 1 CUDA devices (Total VRAM: 24075 MiB):",
            "select with: --backend <cuda|hip|vulkan|metal|cpu> --device <index>",
        ] {
            assert!(cuda_devices(line).is_empty(), "{line}");
        }
    }
}
