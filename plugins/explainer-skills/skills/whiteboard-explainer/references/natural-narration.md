# Natural narration: writing a script that doesn't sound like AI

Read this when you write or review `narration/script.py`. Then run `scripts/aispeak_scan.py` on the script
and fix what it finds before building the voice.

Sources, distilled for a spoken teaching script:
- Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), WikiProject AI Cleanup.
- [blader/humanizer](https://github.com/blader/humanizer), about 61k stars in Oct 2026, based on that page.
- [hardikpandya/stop-slop](https://github.com/hardikpandya/stop-slop), about 18k stars.

For prose that isn't narration (a README, a post), install humanizer itself:
`git clone https://github.com/blader/humanizer ~/.claude/skills/humanizer`.

## Why AI text sounds like AI

A model picks the choice that fits the widest range of readers. The result is staged importance, rhythm by rule,
inflated stakes and lines written to be quoted. A person explaining something to a friend says the fact and moves on.
The word-level tells ("delve", "pivotal", "testament") change with each model generation and are easy to avoid. The
structural ones survive, and in narration they are worse than on a page: the voice gives every staged line the same
dramatic lift, so the listener hears the pattern repeat.

## The tells, strongest first

Act on one sighting of 1-5. For 6-9, act only when the same pattern repeats or other tells sit nearby.

1. **Staged run-ups.** These announce a point instead of making it:
   - "Here's the strange thing." "Here's the intuition." "Here's the idea that changed everything."
   - "Now, the big question." "The key picture is this." "Now here's the crux." "And here's a twist."
   - "Because the details matter."

   Start with the point itself. One "here's" in a whole video is fine; ten is a tic.

2. **Not X but Y, when nobody said X.** These are reframe formulas:
   - "The problem isn't access. The problem is..."
   - "A bigger city isn't harder. It's easier."
   - "It doesn't just say X. It says Y."
   - "The microscope was broken, not the network."

   State Y. Keep a contrast only when X is something the viewer really believes, or both halves carry facts. Quiz
   traps ("present is not the same as used") and a real correction ("It's not a cat and car neuron. It's a V
   neuron.") qualify.

3. **Quotable closers.** A last line that restates the beat as a slogan:
   - "Ablation turns a story into an experiment."
   - "The mess was a learning failure, not a law of nature."
   - "This is science happening in public."
   - "And that's the honest state of things."

   Cut it, or replace it with the next concrete fact. A closer that adds a consequence the viewer hasn't heard can stay.

4. **Dramatic fragments in a row.** For example, "No interpretability tricks. Just an architecture that learns
   better." Merge them into one sentence with a specific claim. A single short line that carries a fact ("Rome
   stays.") is fine.

5. **Staged candor and filler sincerity.** For example, "Honestly, nobody knows yet." or "Let's be clear."
   Drop the word and keep the claim.

6. **Questions the narrator answers at once.** For example, "So what is one neuron, concretely? Let's zoom in."
   A teaching video may ask a few real questions that the next beat builds toward. More than about one per 3 minutes
   sounds like a template. Turn most into statements: "Zoom into a single neuron."

7. **Inflated stakes.**
   - Overclaims: "the best laboratory in all of science", "the strongest test there is".
   - Mood words: "scary conclusion", "the big worry", "astronomically... exponentially many".

   Say the size or the consequence instead, or attribute the claim to the person who made it.

8. **Meta-narration.** For example, "As we'll see", "That's the problem this whole video is about", "Keep this
   picture in mind". One roadmap and one or two "remember X" callbacks to earlier material help a long video. Beyond
   that, cut them.

9. **Adverb crutches.** For example, "actually", "really", "just", "simply", "exactly", "truly". Cut them unless
   they carry meaning. "Just two numbers" is a real contrast with billions of weights; "actually computes" usually isn't.

## Not tells in a narrated explainer (leave them)

- **Connectors at the start of a beat** ("So,", "Now,", "But,", "And"). This style uses them on purpose, so a
  turn in the argument is audible. Vary them, and don't open three beats in a row with "So".
- **Threes that are real data.** "Arabic script, DNA, Base64" are three real features from the paper.
- **Teaching repetition.** A short reminder of a symbol before it is used again is required by the main skill.
- **Colons, commas and periods.** Em dashes are a weak tell. Since Sept 2026 the newest models avoid them, and
  Wikipedia now lists low punctuation variety (only commas and periods) as its own tell. In narration, punctuation
  only shapes the pauses (`shape()` in build_vo.py), so pick it for the pause you want.
- **Opinions the source author holds, attributed to them.** For example, "He calls it a working hypothesis." That
  is reporting, not inflation.

## What human narration does instead

- **Lead with the concrete thing:** the number, the example, the name. "Cut the weakest 75%, and accuracy moves
  from 72 to 71" lands harder than any framing line around it.
- **Vary sentence length.** Follow a long explanatory sentence with a short factual one. Don't make every beat end
  on a punch.
- **Let a surprising fact be surprising** without telling the viewer it is surprising.
- **Keep a little of the speaker:** an aside, a hedge where real doubt exists, a plain "we don't know yet".

## Rewriting a voiced video

Every beat is a TTS line, and scenes are keyed to words in it (`c.w(beat, "word")`). When you reword a line:
- Keep the beat count.
- Keep each trigger word, or re-key the scene to a word the new line still says.
- Run `words.py` until it reports "0 bad word cues".
- Re-voice only the changed lines: build_vo caches per line.
- Re-audit them with `audit_vo.py`.
- Look at a strip of every scene whose lines changed.
