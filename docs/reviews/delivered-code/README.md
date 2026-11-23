# Reviews Of Delivered Code

What the reviews of the code a phase, a roadmap or a session delivered keep in the repository:
the debt register and the recorded mutants, which each review reads before judging the code.

## Documents

- [register.json](register.json) — the debt register: each finding left open, kept or rejected,
  with its axis, its effect, the review that found it and the reason; read it before reopening a
  finding or changing code a review judged.
- [mutants.json](mutants.json) — the recorded mutants of ai-ctl's code, each with the test that
  kills it, which each review runs again; read it before removing or renaming one of those
  tests.
