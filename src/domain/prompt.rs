//! The prompts of the bench: a coding agent at work on a codebase, as
//! OpenCode or a sub-agent sends them, so the model generates what an agent
//! generates — code that reuses the names and lines it has read, which is
//! what speculative decoding predicts well.
//!
//! The code is ai-ctl's own, frozen at one commit so that two runs, even of
//! two versions of ai-ctl, send the same tokens, and a model generates the
//! same reply to them. The request is fictional.

use crate::domain::bench::Message;

/// ai-ctl's Rust sources as of 2026-10-11, at the commit
/// `feat(bench): stream and time chat completions`, each file in a
/// `<file path="…">` block: the code the agent has read. The files the
/// request needs come first — the command line, `main.rs`, `up`'s rendering
/// and its use case — so a prompt of any depth holds them, then the others
/// by path.
pub const CORPUS: &str = include_str!("../../assets/bench/corpus.txt");

/// The agent's system prompt.
const SYSTEM: &str = "\
You are a coding agent working in a Rust repository on the user's machine. You read code, change \
it, run commands and report what you did. You work alone until the task is done: do not ask \
questions you can answer by reading the code.

# Repository

The repository is ai-ctl, a command-line tool that diagnoses a Linux machine with an NVIDIA GPU, \
builds the ggml backends from source, runs llama-swap as the single gateway to the local models, \
and benchmarks the stack. It is a single crate in four layers:

- `src/domain/`: entities and rules, with no I/O;
- `src/application/`: use cases, and the ports (traits) they need for the outside world;
- `src/infrastructure/`: adapters implementing the ports, where commands, files and HTTP happen;
- `src/presentation/`: command-line parsing and rendering.

`main.rs` is the only composition root. Presentation never imports infrastructure. A port exists \
only when two implementations exist or a use case needs a test double.

# Rules

- Errors: `thiserror` in the domain when callers branch on them, `anyhow` with `.context()` \
elsewhere. Never `.unwrap()` outside tests.
- Every change comes with tests: unit tests in the module, parsers tested on captured outputs, use \
cases tested against fakes of their ports. No test depends on timing, a GPU or the network.
- Match the surrounding code: its naming, its comment density, its idioms. Doc comments say what a \
thing is for, in full sentences.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must pass.
- Keep changes minimal: no refactoring the task does not need, no new dependency unless asked.

# Tools

You read files with `read`, which returns a file's content in a `<file path=\"…\">` block. You \
change files with `edit`, giving the path, the exact text to replace and its replacement. You run \
commands with `bash`. When you write code in your answer, give each changed item whole, under the \
path of its file.";

/// The task the agent works on.
const REQUEST: &str = "\
`ai-ctl up` prints its steps on stderr and the gateway's address on stdout. Scripts that start \
the gateway only need the address. Add a `--quiet` option to `ai-ctl up`: with it, `up` prints \
nothing but the address once the gateway is ready, and still prints errors on stderr. Wire it \
through the presentation layer, change the use case only if it must, and add tests for both \
outputs.

I have opened the code you will need. Here it is.";

/// What the agent is asked once it has read the code.
const INSTRUCTION: &str = "\
You have read the code above. Implement the change now: write the complete new version of every \
function, type and test you change or add, each under the path of its file, then say in two \
sentences how you checked it. Do not summarise the files.";

/// A later turn of a session, before the next part of the code.
const TURN: &str = "Output of `read`, the next part of the code:";

/// The text of the corpus, split as the server's tokenizer splits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corpus {
    /// Each token, as the bytes it stands for.
    pieces: Vec<Vec<u8>>,
}

/// Why a tokenized corpus cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the corpus has no token")]
pub struct EmptyCorpus;

impl Corpus {
    pub fn new(pieces: Vec<Vec<u8>>) -> Result<Self, EmptyCorpus> {
        if pieces.is_empty() {
            Err(EmptyCorpus)
        } else {
            Ok(Self { pieces })
        }
    }

    /// The text of `count` tokens from token `start`, the corpus starting
    /// over when it runs out. A character cut by the end of the range is
    /// replaced.
    pub fn text(&self, start: usize, count: usize) -> String {
        let bytes: Vec<u8> = (start..start + count)
            .flat_map(|index| self.pieces[index % self.pieces.len()].iter().copied())
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// The messages of an agent that has read `code`: its system prompt, the
/// request with the code, and the instruction.
pub fn agent(code: &str) -> Vec<Message> {
    vec![
        Message::system(SYSTEM),
        Message::user(format!("{REQUEST}\n\n{code}\n\n{INSTRUCTION}")),
    ]
}

/// A later turn of a session: the next part of the code, `code`.
pub fn turn(code: &str) -> Message {
    Message::user(format!("{TURN}\n\n{code}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::bench::Role;

    fn corpus(pieces: &[&str]) -> Corpus {
        Corpus::new(pieces.iter().map(|p| p.as_bytes().to_vec()).collect()).unwrap()
    }

    #[test]
    fn the_corpus_is_ai_ctls_code_in_file_blocks() {
        assert!(CORPUS.starts_with("<file path=\"src/presentation/cli.rs\">\n"));
        assert!(CORPUS.ends_with("</file>\n\n"));
        let files = CORPUS.matches("<file path=\"src/").count();
        assert_eq!(files, CORPUS.matches("</file>\n").count());
        assert_eq!(files, 47);
    }

    #[test]
    fn a_range_of_tokens_gives_their_text() {
        let corpus = corpus(&["fn", " main", "()", " {", " }"]);
        // Five tokens: the sixth is the first again.
        assert_eq!(corpus.text(0, 6), "fn main() { }fn");
        assert_eq!(corpus.text(0, 3), "fn main()");
        assert_eq!(corpus.text(3, 0), "");
    }

    #[test]
    fn the_corpus_starts_over_when_it_runs_out() {
        let corpus = corpus(&["a", "b", "c"]);
        assert_eq!(corpus.text(1, 6), "bcabca");
    }

    #[test]
    fn a_character_split_across_tokens_is_whole_inside_a_range() {
        // "é" is 0xC3 0xA9, here in two tokens.
        let corpus = Corpus::new(vec![b"h".to_vec(), vec![0xC3], vec![0xA9]]).unwrap();
        assert_eq!(corpus.text(0, 3), "hé");
        assert_eq!(corpus.text(0, 2), "h\u{FFFD}");
    }

    #[test]
    fn a_corpus_needs_a_token() {
        assert_eq!(Corpus::new(Vec::new()), Err(EmptyCorpus));
    }

    #[test]
    fn an_agent_has_read_the_code_before_its_instruction() {
        let messages = agent("<file path=\"src/main.rs\">\nfn main() {}\n</file>");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::System);
        assert!(messages[0].content.starts_with("You are a coding agent"));
        assert_eq!(messages[1].role, Role::User);
        let user = &messages[1].content;
        let code = user.find("<file path=\"src/main.rs\">").unwrap();
        assert!(user[..code].contains("Add a `--quiet` option to `ai-ctl up`"));
        assert!(user[code..].ends_with("Do not summarise the files."));
    }

    #[test]
    fn a_turn_brings_the_next_part_of_the_code() {
        let message = turn("fn next() {}");
        assert_eq!(message.role, Role::User);
        assert!(
            message
                .content
                .ends_with("the next part of the code:\n\nfn next() {}")
        );
    }
}
