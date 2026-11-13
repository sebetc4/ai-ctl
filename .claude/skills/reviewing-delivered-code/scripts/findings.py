#!/usr/bin/env python3
"""Keep a review's findings, checked, in its findings.json, and write its report.md from them.

Usage:
  findings.py new <folder> --base <commit> [--end <commit>] [--roadmap <name> [--phase <n>]]
                  --by <who> --text <go-ahead> --tier <tier> --agent <type>:<effort> [--agent ...]
  findings.py add <folder> <agent> <answer-file>
  findings.py merge <folder> <number> <number> [<number> ...]
  findings.py set <folder> <number>[,<number>...] <field>=<value> [<field>=<value> ...]
  findings.py report <folder>
  findings.py tasks <folder> [<batch>]

<folder> is the review's folder, docs/reviews/delivered-code/<date>-<scope>/, named after
the review. Its findings.json is the source: every command that changes it checks it whole,
refuses a change that breaks one of its rules, then writes it and report.md, whose frame is
assets/report.md.

new     starts the review: its scope from <base> to <end>, HEAD by default, its hunks and
        lines of code as scripts/scope.py counts them, of the kind phase with --roadmap and
        --phase, roadmap with --roadmap alone, range otherwise; the go-ahead; each run of an
        agent, not started, a type given once per run.
add     reads an agent's answer: a `FINDINGS: <n>` line, then n FINDING blocks, any PROBE
        blocks, FILES READ, LEFT OUT, and FACTS, the facts weighed with no finding; any other
        block, such as a verifier's verdicts, is left to the session. A field is one line,
        `name: value`, which an indented line continues. Any fault refuses the whole answer,
        each fault named with its block and its field. Each finding gets the next number;
        its origin, git blame's on the first place of `where`, the defect's site, against the
        scope's base; its floor; its fingerprint; and its matches, the register's entries of
        other reviews sharing one of its axes, a path and a symbol, each printed for the
        session to compare the titles. The first run of the agent's type still to answer
        becomes answered, with what it read, what it left out and its facts; `session`,
        once its runs have answered, answers again in a run of its own.
merge   merges the later findings into the first, their axes, reviewers, places and matches
        added, and numbers the findings again; refused once a verdict or an action is set,
        since repro/<n>/ and the batches cite the numbers.
set     sets fields of findings: severity, effect, verdict, action, batch, note, fixed_in;
        an empty value clears one. The action `known`, a defect the register holds already,
        needs a match and a note, and no verdict. Each finding named has its entry in the
        register while its action is `register` or `rejected`.
report  checks findings.json, after a change made by hand, and writes both files again.
        Each command that reads findings.json refuses a check naming a path another clone lacks,
        one git ignores or one a link leads out of the repository, and any text naming a path
        into a scratch folder, a session's files or the working notes, which no other clone
        opens, such as a reviewer's FILES READ.
tasks   prints the findings of each batch, or of one, as a roadmap's tasks with their
        Proof: lines.

The register, register.json beside the review's folder, is one for the repository: an entry
per finding registered or rejected, its fingerprint, first axis, effect, title, state, review,
number and reason, its note. The state is `rejected`, `kept` for a finding whose fix is
`none:`, `open` otherwise.

Exit codes: 0 done; 1 refused, each fault of the answer, of findings.json or of the request
on stderr; 2 wrong arguments.
"""

import argparse
import json
import os
import re
import shlex
import sys
from pathlib import Path

# Importing scope.py leaves no __pycache__ in the skill's folder.
sys.dont_write_bytecode = True
import scope  # noqa: E402

TEMPLATE = Path(__file__).resolve().parent.parent / "assets" / "report.md"
AXES = ("architecture", "dead and legacy code", "refactoring", "conventions", "bugs", "error handling", "security",
        "tests", "performance", "on-disk formats", "interruption and concurrency", "the CLI's contract", "dependencies")
EFFECTS = ("data lost", "wrong action", "misleading", "slow", "maintenance", "none")
RATED = ("data lost", "wrong action", "misleading")
BELOW = ("maintenance", "none")
SEVERITIES = ("critical", "high", "medium", "low")
ORIGINS = ("new", "older", "mixed")
VERDICTS = ("CONFIRMED", "REFUTED", "UNCERTAIN")
ACTIONS = ("fix", "fix at once", "register", "rejected", "known")
FIXES = ("fix", "fix at once")
REGISTERED = ("register", "rejected")
NOTES = {"register": "the reason the register keeps", "rejected": "the reason the register keeps",
         "known": "the register's entry it is"}
STAGES = ("reviewing", "verification", "fixing", "complete")
STATES = ("not started", "running", "answered")
SESSION = "session"
BLOCK =("axis", "severity", "where", "title", "cause", "effect", "consequence", "fix", "test", "evidence")
TEXTS = ("title", "cause", "consequence", "fix", "test", "evidence")
PROBE = ("command", "risk", "settles", "finding")
SETTABLE = ("severity", "effect", "verdict", "action", "batch", "note", "fixed_in")
REVIEW_KEYS = ("review", "scope", "go_ahead", "agents", "checks", "mutants", "status", "findings")
SCOPE_KEYS = {"phase": ("kind", "roadmap", "phase", "base", "end", "hunks", "lines"),
              "roadmap": ("kind", "roadmap", "base", "end", "hunks", "lines"),
              "range": ("kind", "base", "end", "hunks", "lines")}
AGENT_KEYS = ("type", "effort", "state", "read", "left_out", "probes", "facts")
MUTANT_KEYS = ("diff", "tested", "missed", "caught", "unviable", "seconds", "given_to")
REGISTER = "register.json"
ENTRY_KEYS = ("fingerprint", "axis", "effect", "title", "state", "review", "finding", "reason")
ENTRY_STATES = ("open", "kept", "rejected")
MATCH_KEYS = ("review", "finding", "state", "title")
FINDING_KEYS = ("number", "axes", "reviewers", "where", "title", "cause", "effect", "severity", "consequence", "fix",
                "test", "evidence", "origin", "floor", "fingerprint", "matches", "verdict", "action", "batch", "note",
                "fixed_in")
SLOTS = ("review", "status", "scope", "go_ahead", "summary", "checks", "agents", "findings", "probes", "facts", "coverage",
         "fix_plan")
PLACE_RE = re.compile(r"(\S+):(\d+)(?:-(\d+))? \(([^()]+)\)")
FIELD_RE = re.compile(r"([a-z_]+):(?: (.*))?$")
COUNT_RE = re.compile(r"FINDINGS:\s*(\d+)")
SLOT_RE = re.compile(r"\{\{(\w+)\}\}")
NO_TEST_RE = re.compile(r"none(:\s*)?")
QUOTES = "`'\"(),;"
# A session's folder, its transcripts, its scratchpad, a folder mktemp made, such as the review's scratch, and the
# working notes, which git ignores.
SCRATCH_RE = re.compile(r"/tmp/claude-\d+/|\.claude/projects/|scratchpad/|/tmp\.[A-Za-z0-9]{10}|(?:^|[/`'\"(])notes/")


class FindingsError(Exception):
    pass


def place(text):
    """(path, first, last, symbol) of a place `path:line (symbol)` or `path:first-last (symbol)`."""
    match = PLACE_RE.fullmatch(text)
    if not match:
        raise ValueError(f"`{text}` is no place `path:line (symbol)`")
    path, first, last, symbol = match.groups()
    return path, int(first), int(last or first), symbol


def split_where(text):
    """The places of a FINDING block's `where`, each checked."""
    places = re.split(r"(?<=\)), ", text)
    for one in places:
        place(one)
    return places


def origin(git, extent, where):
    """`new`, `older` or `mixed`: git blame's origin of the first place, the defect's site."""
    path, first, last, _ = place(where[0])
    return scope.origin(git, extent["base"], extent["end"], [(path, first, last)])


def floor(effect, severity):
    """`above` for an effect the user meets with its severity, `below` otherwise."""
    return "above" if severity and effect not in BELOW else "below"


def fingerprint(axes, where):
    """The first axis, the paths and the symbols, free of any line number."""
    places = [place(text) for text in where]
    return (f"{axes[0]} | {' + '.join(sorted({p[0] for p in places}))} | "
            f"{' + '.join(sorted({p[3] for p in places}))}")


def matches(entries, review, axes, where):
    """The register's entries of other reviews the finding may be: one of its axes, a path and a symbol shared."""
    places = [place(text) for text in where]
    paths, symbols = {p[0] for p in places}, {p[3] for p in places}
    found = []
    for known in entries:
        _, their_paths, their_symbols = known["fingerprint"].split(" | ")
        if (known["review"] != review and known["axis"] in axes and paths & set(their_paths.split(" + "))
                and symbols & set(their_symbols.split(" + "))):
            found.append({key: known[key] for key in MATCH_KEYS})
    return found


def entry(review, finding):
    """The register's entry of a finding registered or rejected, None for another."""
    action = finding["action"]
    if action not in REGISTERED:
        return None
    state = "rejected" if action == "rejected" else "kept" if finding["fix"].startswith("none:") else "open"
    return {"fingerprint": finding["fingerprint"], "axis": finding["axes"][0], "effect": finding["effect"],
            "title": finding["title"], "state": state, "review": review, "finding": finding["number"],
            "reason": finding["note"]}


def filed(register, review, numbers):
    """The register with the entries of the findings `numbers` following their action."""
    by = {finding["number"]: finding for finding in review["findings"]}
    others = [known for known in register["entries"]
              if known["review"] != review["review"] or known["finding"] not in numbers]
    return {"entries": others + list(filter(None, (entry(review["review"], by[number]) for number in numbers)))}


def rated(effect, severity):
    """The faults of a severity against its effect."""
    if severity is not None and severity not in SEVERITIES:
        return [f"`severity`: `{severity}` is not one of {', '.join(SEVERITIES)}"]
    if effect in RATED and not severity:
        return [f"`severity` is missing for the effect `{effect}`"]
    if effect in BELOW and severity:
        return [f"`severity` is given for the effect `{effect}`, below the floor"]
    return []


def untested(test):
    """The fault of a `test` that says `none` without its reason."""
    return ["`test`: `none` needs its reason, `none: <why>`"] if NO_TEST_RE.fullmatch(test) else []


def sections(text):
    """The answer's `FINDINGS:` counts, its blocks as (kind, lines), and its FILES READ, LEFT OUT and FACTS lines."""
    counts, blocks, listed = [], [], {}
    current = None
    for line in text.splitlines():
        stripped = line.strip()
        if stripped in ("FINDING", "PROBE"):
            current = []
            blocks.append((stripped, current))
        elif stripped in ("FILES READ", "LEFT OUT", "FACTS"):
            current = listed.setdefault(stripped, [])
        elif not stripped or stripped.startswith("```"):
            current = None
        elif current is not None:
            current.append(line.rstrip())
        elif COUNT_RE.fullmatch(stripped):
            counts.append(int(COUNT_RE.fullmatch(stripped).group(1)))
    return counts, blocks, listed


def fields_of(lines, known):
    """The fields of a block's lines, and their faults."""
    fields, faults, last = {}, [], None
    for line in lines:
        if line[:1].isspace():
            if last is None:
                faults.append(f"line `{line.strip()}` continues no field")
            else:
                fields[last] = f"{fields[last]} {line.strip()}".strip()
            continue
        match = FIELD_RE.match(line)
        last = None
        if not match:
            faults.append(f"line `{line}` is no field")
        elif match.group(1) not in known:
            faults.append(f"`{match.group(1)}` is no field of the block")
        elif match.group(1) in fields:
            faults.append(f"`{match.group(1)}` is given twice")
        else:
            last = match.group(1)
            fields[last] = (match.group(2) or "").strip()
    return fields, faults


def block_faults(fields):
    """The faults of a FINDING block's fields."""
    faults = [f"`{name}` is missing" if name not in fields else f"`{name}` is empty"
              for name in BLOCK if name != "severity" and not fields.get(name)]
    for axis in filter(None, (axis.strip() for axis in fields.get("axis", "").split(","))):
        if axis not in AXES:
            faults.append(f"`axis`: `{axis}` is no axis")
    effect = fields.get("effect")
    if effect and effect not in EFFECTS:
        faults.append(f"`effect`: `{effect}` is not one of {', '.join(EFFECTS)}")
    faults += rated(effect, fields.get("severity"))
    faults += untested(fields["test"]) if fields.get("test") else []
    if fields.get("where"):
        try:
            split_where(fields["where"])
        except ValueError as error:
            faults.append(f"`where`: {error}")
    return faults


def listing(values):
    """The entries of a `in full:` or `in part:` line, `none` holding none."""
    entries = [entry.strip() for entry in re.split(r",\s*(?![^()]*\))", values) if entry.strip()]
    return [] if entries == ["none"] else entries


def read_answer(agent, text):
    """The FINDING blocks' fields, the PROBE blocks', what the agent read, what it left out and its facts."""
    counts, blocks, listed = sections(text)
    faults, found, probes = [], [], []
    for kind, lines in blocks:
        fields, here = fields_of(lines, BLOCK if kind == "FINDING" else PROBE)
        if kind == "FINDING":
            found.append(fields)
            name = f"FINDING {len(found)}" + (f" ({fields['title']})" if fields.get("title") else "")
            here += block_faults(fields)
        else:
            probes.append(fields)
            name = f"PROBE {len(probes)}"
            here += [f"`{field}` is missing" for field in PROBE if not fields.get(field)]
        faults += [f"{agent}: {name}: {fault}" for fault in here]
    titles = [fields.get("title") for fields in found]
    for k, probe in enumerate(probes, 1):
        if probe.get("finding") and probe["finding"] not in titles:
            faults.append(f"{agent}: PROBE {k}: `finding`: `{probe['finding']}` is the title of no FINDING of the answer")
    if len(counts) != 1:
        faults.append(f"{agent}: the answer gives {len(counts)} `FINDINGS: <n>` lines, not one")
    elif counts[0] != len(found):
        faults.append(f"{agent}: the answer gives FINDINGS: {counts[0]} and holds {len(found)} FINDING blocks")
    read = {"in full": [], "in part": []}
    if "FILES READ" not in listed:
        faults.append(f"{agent}: the answer has no FILES READ section")
    for line in listed.get("FILES READ", []):
        name, _, values = line.partition(":")
        if name.strip() in read:
            read[name.strip()] += listing(values)
        else:
            faults.append(f"{agent}: FILES READ: line `{line}` is neither `in full:` nor `in part:`")
    left = [line.strip() for line in listed.get("LEFT OUT", [])]
    facts = [line.strip() for line in listed.get("FACTS", [])]
    if faults:
        raise FindingsError("\n".join(faults))
    return found, probes, read, [] if left == ["none"] else left, facts


def add(review, agent, text, git, entries):
    """Adds an agent's answer to the review, to the first run of its type still to answer, or to a new run of the
    session's once its runs have answered; a line per finding the register's `entries` may hold."""
    runs = [a for a in review["agents"] if a["type"] == agent]
    if not runs:
        raise FindingsError(f"{agent} is no agent of the review: {', '.join(a['type'] for a in review['agents'])}")
    entry = next((a for a in runs if a["state"] != "answered"), None)
    # A defect the session sees after its answer, in the triage or a fix, costs no agent the go-ahead counted.
    later = entry is None and agent == SESSION
    if later:
        entry = {"type": SESSION, "effort": runs[0]["effort"], "state": "not started"}
    if entry is None:
        raise FindingsError(f"{agent} has answered already, each of its {plural(len(runs), 'run')}")
    found, probes, read, left, facts = read_answer(agent, text)
    faults, numbers, lines = [], {}, []
    for k, fields in enumerate(found, 1):
        where = split_where(fields["where"])
        try:
            scope.origin(git, review["scope"]["base"], review["scope"]["end"],
                         [place(text)[:3] for text in where])
            first = origin(git, review["scope"], where)
        except scope.ScopeError as error:
            faults.append(f"{agent}: FINDING {k} ({fields['title']}): `where`: {error}")
            continue
        axes = [axis.strip() for axis in fields["axis"].split(",")]
        severity = fields.get("severity") or None
        number = len(review["findings"]) + 1
        numbers[fields["title"]] = number
        held = matches(entries, review["review"], axes, where)
        lines += [f"finding {number}: the register holds {m['review']} finding {m['finding']}, {m['state']}: "
                  f"{m['title']}" for m in held]
        review["findings"].append({
            "number": number, "axes": axes, "reviewers": [agent], "where": where, "title": fields["title"],
            "cause": fields["cause"], "effect": fields["effect"], "severity": severity,
            "consequence": fields["consequence"], "fix": fields["fix"], "test": fields["test"],
            "evidence": fields["evidence"], "origin": first, "floor": floor(fields["effect"], severity),
            "fingerprint": fingerprint(axes, where), "matches": held, "verdict": None, "action": None, "batch": None,
            "note": None, "fixed_in": None})
    if faults:
        raise FindingsError("\n".join(faults))
    if later:
        review["agents"].append(entry)
    entry.update(state="answered", read=read, left_out=left)
    if facts:
        entry["facts"] = facts
    if probes:
        entry["probes"] = [{**{key: probe[key] for key in PROBE[:3]}, "finding": numbers[probe["finding"]]}
                           for probe in probes]
    return lines


def renumber(review, numbers):
    """Numbers the findings again, `numbers` mapping each old number to the finding it went to."""
    new = {}
    for index, finding in enumerate(review["findings"], 1):
        new[finding["number"]] = index
        finding["number"] = index
    for agent in review["agents"]:
        for probe in agent.get("probes", []):
            probe["finding"] = new[numbers.get(probe["finding"], probe["finding"])]


def merge(review, numbers):
    """Merges the findings `numbers[1:]` into `numbers[0]`."""
    by = {finding["number"]: finding for finding in review["findings"]}
    missing = [str(n) for n in numbers if n not in by]
    if missing:
        raise FindingsError(f"no finding {', '.join(missing)}")
    if len(set(numbers)) != len(numbers):
        raise FindingsError("a finding is named twice")
    if any(finding["verdict"] or finding["action"] for finding in review["findings"]):
        raise FindingsError("the triage has begun: repro/<n>/ and the batches cite the findings' numbers")
    into = by[numbers[0]]
    for number in numbers[1:]:
        for key in ("axes", "reviewers", "where", "matches"):
            into[key] += [value for value in by[number][key] if value not in into[key]]
    into["fingerprint"] = fingerprint(into["axes"], into["where"])
    review["findings"] = [finding for finding in review["findings"] if finding["number"] not in numbers[1:]]
    renumber(review, {number: numbers[0] for number in numbers[1:]})


def assign(review, numbers, values):
    """Sets the fields `values`, each `field=value`, of the findings `numbers`."""
    by = {finding["number"]: finding for finding in review["findings"]}
    missing = [str(n) for n in numbers if n not in by]
    if missing:
        raise FindingsError(f"no finding {', '.join(missing)}")
    changes = {}
    for text in values:
        name, sep, value = text.partition("=")
        if not sep or name not in SETTABLE:
            raise FindingsError(f"`{name}` is no field to set: {', '.join(SETTABLE)}")
        if name == "batch" and value and not value.isdigit():
            raise FindingsError(f"`batch`: `{value}` is no number")
        changes[name] = (int(value) if name == "batch" else value) if value else None
    for number in numbers:
        finding = by[number]
        finding.update(changes)
        finding["floor"] = floor(finding["effect"], finding["severity"])


def keys(name, value, wanted, optional=()):
    """The faults of a mapping's keys against those wanted."""
    if not isinstance(value, dict):
        return [f"{name} is no mapping"]
    faults = [f"{name}: `{key}` is missing" for key in wanted if key not in value]
    return faults + [f"{name}: `{key}` is no key of it" for key in value if key not in wanted and key not in optional]


def text_faults(name, value, empty=False):
    if value is None and empty:
        return []
    if not isinstance(value, str) or not value.strip():
        return [f"{name} is no text"]
    return [f"{name} holds a line break"] if "\n" in value else []


def scratch_faults(name, value):
    """Each text of `value` naming a path into a scratch folder, a session's files or the working notes, which no
    other clone opens."""
    if isinstance(value, dict):
        return [fault for key, item in value.items() for fault in scratch_faults(f"{name}: `{key}`", item)]
    if isinstance(value, list):
        return [fault for k, item in enumerate(value) for fault in scratch_faults(f"{name}[{k}]", item)]
    if not isinstance(value, str):
        return []
    return [f"{name} names `{token.strip(QUOTES).rstrip('.')}`, a path into a scratch folder, a session's files or "
            "the working notes, which no other clone opens: name a scratch file by its place in the review's "
            "scratch folder, such as `scope.txt`, and anything else by what it holds"
            for token in value.split() if SCRATCH_RE.search(token)]


def number_faults(name, value, least=0):
    ok = isinstance(value, int) and not isinstance(value, bool) and value >= least
    return [] if ok else [f"{name} is no number of {least} or more"]


def list_faults(name, value, each):
    if not isinstance(value, list):
        return [f"{name} is no list"]
    return [fault for k, item in enumerate(value) for fault in each(f"{name}[{k}]", item)]


def scope_faults(extent):
    if not isinstance(extent, dict) or extent.get("kind") not in SCOPE_KEYS:
        return [f"scope: `kind` is not one of {', '.join(SCOPE_KEYS)}"]
    faults = keys("scope", extent, SCOPE_KEYS[extent["kind"]])
    if faults:
        return faults
    faults += text_faults("scope: `base`", extent["base"], empty=True) + text_faults("scope: `end`", extent["end"])
    faults += number_faults("scope: `hunks`", extent["hunks"])
    if "roadmap" in extent:
        faults += text_faults("scope: `roadmap`", extent["roadmap"])
    if "phase" in extent:
        faults += number_faults("scope: `phase`", extent["phase"])
    lines = extent["lines"]
    faults += keys("scope: `lines`", lines, ("added", "removed"))
    if isinstance(lines, dict):
        faults += [fault for key in ("added", "removed") if key in lines
                   for fault in number_faults(f"scope: `lines`: `{key}`", lines[key])]
    return faults


def agent_faults(name, agent):
    faults = keys(name, agent, AGENT_KEYS[:3], AGENT_KEYS[3:])
    if faults:
        return faults
    faults += text_faults(f"{name}: `type`", agent["type"]) + text_faults(f"{name}: `effort`", agent["effort"])
    if agent["state"] not in STATES:
        faults.append(f"{name}: `state` is not one of {', '.join(STATES)}")
    if "read" in agent:
        faults += keys(f"{name}: `read`", agent["read"], ("in full", "in part"))
        if isinstance(agent["read"], dict):
            faults += [fault for key in ("in full", "in part")
                       for fault in list_faults(f"{name}: `read`: `{key}`", agent["read"].get(key, []), text_faults)]
    if "left_out" in agent:
        faults += list_faults(f"{name}: `left_out`", agent["left_out"], text_faults)
    if "facts" in agent:
        faults += list_faults(f"{name}: `facts`", agent["facts"], text_faults)
    if "probes" in agent:
        def probe_faults(label, probe):
            found = keys(label, probe, PROBE)
            return found or [fault for key in PROBE[:3] for fault in text_faults(f"{label}: `{key}`", probe[key])] + \
                number_faults(f"{label}: `finding`", probe["finding"], 1)
        faults += list_faults(f"{name}: `probes`", agent["probes"], probe_faults)
    return faults


def match_faults(name, value):
    faults = keys(name, value, MATCH_KEYS)
    if faults:
        return faults
    faults += text_faults(f"{name}: `review`", value["review"]) + text_faults(f"{name}: `title`", value["title"])
    faults += number_faults(f"{name}: `finding`", value["finding"], 1)
    return faults + ([] if value["state"] in ENTRY_STATES else
                     [f"{name}: `state` is not one of {', '.join(ENTRY_STATES)}"])


def entry_faults(name, value):
    faults = keys(name, value, ENTRY_KEYS)
    if faults:
        return faults
    faults += [fault for key in ("fingerprint", "title", "review", "reason")
               for fault in text_faults(f"{name}: `{key}`", value[key])]
    faults += number_faults(f"{name}: `finding`", value["finding"], 1)
    if value["axis"] not in AXES:
        faults.append(f"{name}: `axis` is no axis")
    if value["effect"] not in EFFECTS:
        faults.append(f"{name}: `effect` is not one of {', '.join(EFFECTS)}")
    if value["state"] not in ENTRY_STATES:
        faults.append(f"{name}: `state` is not one of {', '.join(ENTRY_STATES)}")
    if isinstance(value["fingerprint"], str):
        parts = value["fingerprint"].split(" | ")
        if len(parts) != 3 or parts[0] != value["axis"]:
            faults.append(f"{name}: `fingerprint` is no `<axis> | <paths> | <symbols>` of its axis")
    return faults


def register_faults(register):
    """Each rule of register.json the register breaks."""
    if not isinstance(register, dict) or list(register) != ["entries"]:
        return ["is no mapping of `entries` alone"]
    faults = list_faults("`entries`", register["entries"], entry_faults)
    if faults:
        return faults
    named = [(known["review"], known["finding"]) for known in register["entries"]]
    return [f"`entries`: {review} finding {number} is listed twice"
            for review, number in sorted({one for one in named if named.count(one) > 1})]


def finding_rules(finding):
    """The faults of a finding's fields, each of whose types is right."""
    name = f"finding {finding['number']}"
    faults = []
    for axis in finding["axes"]:
        if axis not in AXES:
            faults.append(f"{name}: `axes`: `{axis}` is no axis")
    try:
        where_ok = [place(text) for text in finding["where"]]
    except ValueError as error:
        faults.append(f"{name}: `where`: {error}")
        where_ok = None
    if finding["effect"] not in EFFECTS:
        faults.append(f"{name}: `effect` is not one of {', '.join(EFFECTS)}")
    faults += [f"{name}: {fault}" for fault in rated(finding["effect"], finding["severity"])]
    faults += [f"{name}: {fault}" for fault in untested(finding["test"])]
    if finding["origin"] not in ORIGINS:
        faults.append(f"{name}: `origin` is not one of {', '.join(ORIGINS)}")
    level = floor(finding["effect"], finding["severity"])
    if finding["floor"] != level:
        faults.append(f"{name}: `floor` is {finding['floor']}, where its effect and severity give {level}")
    if where_ok and finding["axes"] and finding["fingerprint"] != fingerprint(finding["axes"], finding["where"]):
        faults.append(f"{name}: `fingerprint` is not the one its axes and places give, "
                      f"`{fingerprint(finding['axes'], finding['where'])}`")
    verdict, action, batch = finding["verdict"], finding["action"], finding["batch"]
    if verdict is not None and verdict not in VERDICTS:
        faults.append(f"{name}: `verdict`: `{verdict}` is not one of {', '.join(VERDICTS)}")
    if verdict and level == "below":
        faults.append(f"{name}: `verdict` is given below the floor, which gets no verification")
    if action is not None and action not in ACTIONS:
        faults.append(f"{name}: `action`: `{action}` is not one of {', '.join(ACTIONS)}")
    if action and action != "known" and level == "above" and not verdict:
        faults.append(f"{name}: `action` is set with no `verdict`, above the floor")
    if action == "known" and not finding["matches"]:
        faults.append(f"{name}: `action` `known` needs an entry of the register in `matches`")
    if action == "rejected" and verdict != "REFUTED" and level == "above":
        faults.append(f"{name}: `action` `rejected` needs the verdict REFUTED, above the floor")
    if verdict == "REFUTED" and action not in (None, "rejected"):
        faults.append(f"{name}: `action` `{action}` follows the verdict REFUTED, where only `rejected` does")
    if action in FIXES and batch is None:
        faults.append(f"{name}: `batch` is missing for the action `{action}`")
    if batch is not None and action not in FIXES:
        faults.append(f"{name}: `batch` is set with no fix")
    if action in NOTES and not finding["note"]:
        faults.append(f"{name}: `note` is missing: {NOTES[action]}")
    if finding["fixed_in"] and action not in FIXES:
        faults.append(f"{name}: `fixed_in` is set with no fix")
    return faults


def finding_faults(index, finding):
    """The faults of the finding at `index` in findings.json: its keys and types, then its rules."""
    label = f"finding at position {index}"
    faults = keys(label, finding, FINDING_KEYS)
    if faults:
        return faults
    if finding["number"] != index:
        return [f"{label}: `number` is {finding['number']}, not {index}"]
    name = f"finding {index}"
    for key in ("axes", "reviewers", "where"):
        if not isinstance(finding[key], list) or not finding[key]:
            faults.append(f"{name}: `{key}` is no list of one or more")
        else:
            faults += list_faults(f"{name}: `{key}`", finding[key], text_faults)
    faults += [fault for key in TEXTS for fault in text_faults(f"{name}: `{key}`", finding[key])]
    faults += [fault for key in ("note", "fixed_in") for fault in text_faults(f"{name}: `{key}`", finding[key], True)]
    faults += list_faults(f"{name}: `matches`", finding["matches"], match_faults)
    for key in ("effect", "severity", "origin", "floor", "fingerprint", "verdict", "action"):
        if finding[key] is not None and not isinstance(finding[key], str):
            faults.append(f"{name}: `{key}` is no text")
    if finding["batch"] is not None:
        faults += number_faults(f"{name}: `batch`", finding["batch"], 1)
    return faults or finding_rules(finding)


def problems(review):
    """Each rule of findings.json the review breaks."""
    faults = keys("findings.json", review, REVIEW_KEYS)
    if faults:
        return faults
    faults += scratch_faults("findings.json", review)
    faults += text_faults("`review`", review["review"]) + scope_faults(review["scope"])
    go_ahead = review["go_ahead"]
    faults += keys("go_ahead", go_ahead, ("by", "text", "tier")) or \
        [fault for key in ("by", "text", "tier") for fault in text_faults(f"go_ahead: `{key}`", go_ahead[key])]
    faults += list_faults("agents", review["agents"], agent_faults)
    faults += list_faults("checks", review["checks"], lambda name, check: keys(name, check, ("command", "result")) or
                          text_faults(f"{name}: `command`", check["command"]) +
                          text_faults(f"{name}: `result`", check["result"]))
    mutants = review["mutants"]
    if mutants is not None:
        faults += keys("mutants", mutants, MUTANT_KEYS) or \
            text_faults("mutants: `diff`", mutants["diff"]) + text_faults("mutants: `given_to`", mutants["given_to"], True) + \
            [fault for key in MUTANT_KEYS[1:6] for fault in number_faults(f"mutants: `{key}`", mutants[key])]
    status = review["status"]
    faults += keys("status", status, ("stage", "note")) or text_faults("status: `note`", status["note"])
    if isinstance(status, dict) and status.get("stage") not in STAGES:
        faults.append(f"status: `stage` `{status.get('stage')}` is not one of {', '.join(STAGES)}")
    if not isinstance(review["findings"], list):
        return faults + ["`findings` is no list"]
    found = [fault for index, finding in enumerate(review["findings"], 1)
             for fault in finding_faults(index, finding)]
    faults += found
    if found:
        return faults
    count = len(review["findings"])
    if isinstance(review["agents"], list):
        for agent in review["agents"]:
            for probe in agent.get("probes", []) if isinstance(agent, dict) else []:
                if isinstance(probe, dict) and isinstance(probe.get("finding"), int) and probe["finding"] > count:
                    faults.append(f"agents: {agent.get('type')}: a probe serves finding {probe['finding']}, "
                                  f"of {count} findings")
    if isinstance(status, dict) and status.get("stage") == "complete":
        for finding in review["findings"]:
            if not finding["action"]:
                faults.append(f"status: stage `complete`, while finding {finding['number']} has no action")
            elif finding["action"] in FIXES and not finding["fixed_in"]:
                faults.append(f"status: stage `complete`, while finding {finding['number']} has no `fixed_in`")
    return faults


def ordered(value, wanted):
    return {key: value[key] for key in wanted if key in value}


def dump(review):
    """findings.json's text: its keys in their order, two spaces of indent, the line separators escaped."""
    canonical = ordered(review, REVIEW_KEYS)
    extent = ordered(review["scope"], SCOPE_KEYS[review["scope"]["kind"]])
    canonical["scope"] = {**extent, "lines": ordered(extent["lines"], ("added", "removed"))}
    canonical["go_ahead"] = ordered(review["go_ahead"], ("by", "text", "tier"))
    canonical["agents"] = []
    for agent in review["agents"]:
        entry = ordered(agent, AGENT_KEYS)
        if "read" in entry:
            entry["read"] = ordered(entry["read"], ("in full", "in part"))
        if "probes" in entry:
            entry["probes"] = [ordered(probe, PROBE) for probe in entry["probes"]]
        canonical["agents"].append(entry)
    canonical["checks"] = [ordered(check, ("command", "result")) for check in review["checks"]]
    canonical["mutants"] = None if review["mutants"] is None else ordered(review["mutants"], MUTANT_KEYS)
    canonical["status"] = ordered(review["status"], ("stage", "note"))
    canonical["findings"] = [{**ordered(finding, FINDING_KEYS),
                              "matches": [ordered(match, MATCH_KEYS) for match in finding["matches"]]}
                             for finding in review["findings"]]
    return as_json(canonical)


def dump_register(register):
    """register.json's text: its entries by review and finding, their keys in their order."""
    entries = sorted(register["entries"], key=lambda known: (known["review"], known["finding"]))
    return as_json({"entries": [ordered(known, ENTRY_KEYS) for known in entries]})


def as_json(value):
    """`value` as JSON with two spaces of indent, the line separators escaped."""
    text = json.dumps(value, indent=2, ensure_ascii=False) + "\n"
    return text.replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")


def sentence(text):
    return text if text.endswith((".", "!", "?")) else f"{text}."


def plural(count, word):
    return f"{count} {word}" if count == 1 else f"{count} {word}s"


def scope_line(extent):
    span = f"{extent['base']}..{extent['end']}" if extent["base"] else f"the history up to {extent['end']}"
    if extent["kind"] == "phase":
        span = f"roadmap `{extent['roadmap']}`, phase {extent['phase']}, {span}"
    elif extent["kind"] == "roadmap":
        span = f"roadmap `{extent['roadmap']}`, {span}"
    lines = extent["lines"]
    return f"{span}: {plural(extent['hunks'], 'hunk')}, +{lines['added']} −{lines['removed']} lines of code"


def summary(findings):
    if not findings:
        return "No finding."
    rows = ["| # | Title | Axis | Effect | Severity | Origin | Verdict | Action | Batch |",
            "|---|---|---|---|---|---|---|---|---|"]
    for f in findings:
        verdict = f["verdict"] or ("to verify" if f["floor"] == "above" and f["action"] != "known" else "—")
        cells = [f["number"], f["title"].replace("|", "\\|"), ", ".join(f["axes"]), f["effect"], f["severity"] or "—",
                 f["origin"], verdict, f["action"] or "—", f["batch"] or "—"]
        rows.append("| " + " | ".join(str(cell) for cell in cells) + " |")
    return "\n".join(rows)


def finding_section(f):
    block = ["FINDING", f"axis: {', '.join(f['axes'])}"] + ([f"severity: {f['severity']}"] if f["severity"] else [])
    block += [f"where: {', '.join(f['where'])}", f"title: {f['title']}", f"cause: {f['cause']}",
              f"effect: {f['effect']}", f"consequence: {f['consequence']}", f"fix: {f['fix']}", f"test: {f['test']}",
              f"evidence: {f['evidence']}"]
    fence = "````" if any("```" in line for line in block) else "```"
    lines = [f"### {f['number']}. {f['title']}", "", fence, *block, fence, "",
             f"Reviewers: {', '.join(f['reviewers'])}. Origin: {f['origin']}. Floor: {f['floor']}. "
             f"Fingerprint: `{f['fingerprint']}`."]
    if f["matches"]:
        lines.append("Register: " + sentence("; ".join(f"`{m['review']}` finding {m['finding']}, {m['state']}: "
                                                       f"{m['title']}" for m in f["matches"])))
    if f["verdict"] or f["action"] or f["note"]:
        verdict = f["verdict"] or ("none, below the floor" if f["floor"] == "below" else
                                   "none, in the register" if f["action"] == "known" else "to verify")
        line = f"Verdict: {verdict}."
        if f["action"]:
            line += f" Action: {f['action']}" + (f", batch {f['batch']}" if f["batch"] else "") + "."
        if f["note"]:
            line += f" {sentence(f['note'])}"
        lines.append(line)
    if f["fixed_in"]:
        lines.append(f"Fixed in `{f['fixed_in']}`.")
    return "\n".join(lines)


def checks_section(review):
    lines = [f"- `{check['command']}`: {check['result']}" for check in review["checks"]]
    m = review["mutants"]
    if m:
        line = (f"- `cargo mutants --in-diff` on {m['diff']}: {m['tested']} mutants in {m['seconds']} s, "
                f"{m['missed']} missed, {m['caught']} caught, {m['unviable']} unviable")
        lines.append(line + (f"; the missed to {m['given_to']}" if m["given_to"] else ""))
    return "\n".join(lines)


def batches(findings):
    """{batch: its findings}, by batch number."""
    by = {}
    for finding in findings:
        if finding["batch"]:
            by.setdefault(finding["batch"], []).append(finding)
    return dict(sorted(by.items()))


def render(review, template):
    """report.md's text: the frame `template`, each `{{part}}` filled, a section whose part is empty left out."""
    findings = review["findings"]
    agents = review["agents"]
    parts = {
        "review": review["review"],
        "status": f"{review['status']['stage']} — {review['status']['note']}",
        "scope": scope_line(review["scope"]),
        "go_ahead": f"{review['go_ahead']['by']}, tier `{review['go_ahead']['tier']}`",
        "summary": summary(findings),
        "checks": checks_section(review),
        "agents": "\n".join(f"- `{a['type']}`, effort `{a['effort']}`: {a['state']}" for a in agents),
        "findings": "\n\n".join(finding_section(f) for f in findings),
        "probes": "\n".join(f"- Finding {p['finding']}, from `{a['type']}`: `{p['command']}`. "
                            f"Risk: {sentence(p['risk'])} Settles: {sentence(p['settles'])}"
                            for a in agents for p in a.get("probes", [])),
        "facts": "\n".join(f"- `{a['type']}`: {fact}" for a in agents for fact in a.get("facts", [])),
        "coverage": "\n".join(f"- `{a['type']}`: in full: {', '.join(a['read']['in full']) or 'none'}; "
                              f"in part: {', '.join(a['read']['in part']) or 'none'}; "
                              f"left out: {'; '.join(a.get('left_out', [])) or 'none'}"
                              for a in agents if "read" in a),
        "fix_plan": "\n".join(f"- Batch {batch}: findings {', '.join(str(f['number']) for f in held)}"
                              for batch, held in batches(findings).items()),
    }
    named = SLOT_RE.findall(template)
    for name in named:
        if name not in parts:
            raise FindingsError(f"the frame names {{{{{name}}}}}, which the writer does not fill")
    for name in parts:
        if named.count(name) != 1:
            raise FindingsError(f"the frame holds {{{{{name}}}}} {named.count(name)} times, not once")
    kept = []
    for section in re.split(r"\n(?=## )", template.rstrip("\n")):
        body = SLOT_RE.fullmatch(section.split("\n", 1)[-1].strip()) if section.startswith("## ") else None
        if not (body and not parts[body.group(1)]):
            kept.append(section.rstrip("\n"))
    return SLOT_RE.sub(lambda match: parts[match.group(1)], "\n\n".join(kept) + "\n")


def tasks(review, batch=None):
    """The findings of each batch, or of `batch`, as a roadmap's tasks with their Proof: lines."""
    by = batches(review["findings"])
    if not by:
        raise FindingsError("no finding has a batch yet")
    if batch is not None and batch not in by:
        raise FindingsError(f"no finding in batch {batch}: the batches are {', '.join(map(str, by))}")
    blocks = []
    for number, held in by.items():
        if batch is not None and number != batch:
            continue
        lines = [f"### Batch {number}"]
        for f in held:
            lines.append(f"- [{'x' if f['fixed_in'] else ' '}] Finding {f['number']}: {f['fix']}")
            proof = f["test"] if f["test"].startswith("none:") else f"{f['test']}, watched failing, then passing"
            lines.append(f"  Proof: {proof}")
        blocks.append("\n".join(lines))
    return "\n\n".join(blocks)


def load(folder):
    path = folder / "findings.json"
    try:
        review = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise FindingsError(f"{path} does not exist: `new` starts a review") from None
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FindingsError(f"{path}: {error}") from None
    faults = problems(review)
    if faults:
        raise FindingsError("\n".join(f"{path}: {fault}" for fault in faults))
    if review["review"] != folder.name:
        raise FindingsError(f"{path} holds the review {review['review']}, not {folder.name}, its folder's name")
    faults = clone_faults(review, folder)
    if faults:
        raise FindingsError("\n".join(f"{path}: {fault}" for fault in faults))
    return review


def clone_faults(review, folder):
    """Each path a check's command names that another clone of the review's repository lacks: one git ignores, or
    one a link leads out of the repository. None outside a repository, which no clone shares."""
    if not review["checks"]:
        return []
    try:
        git = scope.repository(folder)
    except scope.ScopeError:
        return []
    root = os.path.realpath(git.folder)
    faults = []
    for index, check in enumerate(review["checks"], 1):
        try:
            words = shlex.split(check["command"])
        except ValueError:
            words = check["command"].split()
        for word in words:
            if word.startswith("-") or "/" not in word or os.path.isabs(word):
                continue
            real = os.path.realpath(os.path.join(root, word))
            if real != root and not real.startswith(root + os.sep):
                why = "a link leads out of the repository"
            elif git.run("check-ignore", "-q", word, check=False).returncode == 0:
                why = "git ignores it"
            else:
                continue
            faults.append(f"check {index}: `{word}` is no path of another clone, {why}: "
                          f"name the program by its file name, `{Path(word).name}`")
    return faults


def load_register(folder):
    """register.json beside the review's folder, with no entry when it does not exist."""
    path = folder.parent / REGISTER
    try:
        register = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {"entries": []}
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FindingsError(f"{path}: {error}") from None
    faults = register_faults(register)
    if faults:
        raise FindingsError("\n".join(f"{path}: {fault}" for fault in faults))
    return register


def write(path, text):
    """Writes `path` whole or leaves it as it was."""
    partial = path.with_name(f".{path.name}.partial")
    partial.write_text(text, encoding="utf-8")
    os.replace(partial, path)


def save(folder, review, register=None):
    """Writes findings.json and report.md, once the review breaks no rule, then the register when given."""
    faults = problems(review)
    if faults:
        raise FindingsError("\n".join(faults))
    if review["review"] != folder.name:
        raise FindingsError(f"the review {review['review']} is not {folder.name}, its folder's name")
    report = render(review, TEMPLATE.read_text(encoding="utf-8"))
    folder.mkdir(parents=True, exist_ok=True)
    write(folder / "findings.json", dump(review))
    write(folder / "report.md", report)
    path = folder.parent / REGISTER
    if register is not None and (register["entries"] or path.exists()):
        write(path, dump_register(register))


def new(args):
    folder = args.folder
    if (folder / "findings.json").exists():
        raise FindingsError(f"{folder / 'findings.json'} exists: the review has started")
    if args.phase is not None and args.roadmap is None:
        raise FindingsError("--phase needs --roadmap")
    anchor = folder
    while not anchor.exists():
        anchor = anchor.parent
    git = scope.repository(anchor)
    base, end = scope.resolve(git, args.base), scope.resolve(git, args.end)
    if not git.is_ancestor(base, end):
        raise FindingsError(f"{base} is not an ancestor of {end}")
    _, _, counts, routed = scope.routes(git, base, end)
    hunks = scope.code_hunks(git, base, end, routed, counts)[0]
    kind = "range" if args.roadmap is None else "roadmap" if args.phase is None else "phase"
    extent = {"kind": kind, "roadmap": args.roadmap, "phase": args.phase, "base": base, "end": end,
              "hunks": len(hunks), "lines": {"added": sum(h[1] for h in hunks), "removed": sum(h[2] for h in hunks)}}
    agents = []
    for text in args.agent:
        name, sep, effort = text.partition(":")
        if not sep or not name or not effort:
            raise FindingsError(f"--agent {text}: give it as <type>:<effort>")
        agents.append({"type": name, "effort": effort, "state": "not started"})
    review = {"review": folder.name, "scope": ordered(extent, SCOPE_KEYS[kind]),
              "go_ahead": {"by": args.by, "text": args.text, "tier": args.tier}, "agents": agents, "checks": [],
              "mutants": None, "status": {"stage": "reviewing", "note": "the reviewers' answers to come"},
              "findings": []}
    save(folder, review)


def numbers(text):
    try:
        return [int(part) for part in text.split(",")]
    except ValueError:
        raise argparse.ArgumentTypeError(f"`{text}` is no list of numbers such as 1,4") from None


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    start = commands.add_parser("new", help="start a review's folder")
    start.add_argument("folder", type=Path)
    start.add_argument("--base", required=True)
    start.add_argument("--end", default="HEAD")
    start.add_argument("--roadmap")
    start.add_argument("--phase", type=int)
    start.add_argument("--by", required=True, help="who gave the go-ahead, such as `the request`")
    start.add_argument("--text", required=True, help="the go-ahead's words")
    start.add_argument("--tier", required=True)
    start.add_argument("--agent", action="append", required=True, help="<type>:<effort>, once per run of an agent")
    answer = commands.add_parser("add", help="add an agent's answer")
    answer.add_argument("folder", type=Path)
    answer.add_argument("agent")
    answer.add_argument("answer", type=Path)
    together = commands.add_parser("merge", help="merge findings into the first")
    together.add_argument("folder", type=Path)
    together.add_argument("numbers", type=int, nargs="+")
    change = commands.add_parser("set", help="set fields of findings")
    change.add_argument("folder", type=Path)
    change.add_argument("numbers", type=numbers)
    change.add_argument("values", nargs="+", help="<field>=<value>")
    again = commands.add_parser("report", help="check findings.json and write the report again")
    again.add_argument("folder", type=Path)
    plan = commands.add_parser("tasks", help="print the batches as a roadmap's tasks")
    plan.add_argument("folder", type=Path)
    plan.add_argument("batch", type=int, nargs="?")
    args = parser.parse_args(argv)
    if args.command == "merge" and len(args.numbers) < 2:
        parser.error("merge takes two numbers or more")
    try:
        if args.command == "new":
            new(args)
            return 0
        review = load(args.folder)
        if args.command == "tasks":
            print(tasks(review, args.batch))
            return 0
        register, shown = None, []
        if args.command == "add":
            try:
                text = args.answer.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError) as error:
                raise FindingsError(f"{args.answer}: {error}") from None
            entries = load_register(args.folder)["entries"]
            shown = add(review, args.agent, text, scope.repository(args.folder), entries)
        elif args.command == "merge":
            merge(review, args.numbers)
        elif args.command == "set":
            register = load_register(args.folder)
            assign(review, args.numbers, args.values)
            register = filed(register, review, args.numbers)
        save(args.folder, review, register)
        for line in shown:
            print(line)
    except (FindingsError, scope.ScopeError) as error:
        for line in str(error).splitlines():
            print(f"findings.py: {line}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
