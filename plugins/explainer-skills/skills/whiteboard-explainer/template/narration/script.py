"""Narration for the template's demo video: "Binary search, in about a minute".

Each scene is one "slide"; each beat is one sentence-sized TTS line (one sentence per beat, ~150 wpm).
The scene names must match the dispatch arms in src/lib.rs (`"Idea" => s_idea::idea(t, c)`).

Spelling is for the voice: numbers are written as words ("fifty-two", "two point one million"), letters
that should be read as letters are spaced ("C P U"), and names the voice gets wrong are respelled.
narration/make_srt.py maps the spellings back for the subtitles (its DISPLAY list).

`PAUSE` = the quiz's thinking time (PAUSE_LEN seconds in build_vo.py, with a countdown on screen).
Quiz keys (every option justified): research/quiz_keys.md in a real project. For this demo:
  worst-case guesses for n sorted entries = floor(log2 n) + 1; n = 1,000,000 -> 20; n = 2,000,000 -> 21.
"""

PAUSE = "__pause__"

SCENES = [
    # ================================================================ cold open (no chapter chip)
    ("Hook", [
        "Imagine a list of a million names, sorted from A to Z.",
        "To find one name, you point at any entry and ask, is my name before it, or after?",
        "How many questions do you need?",
        "Twenty is always enough.",
        "The trick is called binary search.",
    ]),
    # ================================================================ 1 · the idea
    ("Idea", [
        "Here's a short sorted list, and we want fifty-two.",
        "First, look at the middle, which is forty.",
        "Fifty-two is bigger, so the left half goes.",
        "Then check the middle of what's left, sixty-three.",
        "Too big, so the right side goes.",
        "The middle of the last three is fifty-two, found in three guesses.",
        "Each guess cuts the list in half.",
    ]),
    # ================================================================ 2 · bigger lists
    ("Figure", [
        "This chart shows the most guesses binary search can need, for each list size.",
        "Ten names take four guesses, and a thousand take ten.",
        "Each time the list gets ten times longer, you add only three or four.",
        "So a million names need just twenty.",
    ]),
    # ================================================================ quiz
    ("Quiz1", [
        "Quiz one.",
        "Your sorted list doubles, from one million names to two million. How many guesses now, at most?",
        "Option A, forty.",
        "Option B, still twenty.",
        "Option C, twenty-one.",
        "Option D, twenty-three.",
        PAUSE,
        "The answer is C.",
        "Doubling adds just one more halving, so twenty-one is enough.",
        "Forty is the tempting one, but that's how checking names one by one grows.",
    ]),
    # ================================================================ outro (no chapter chip)
    ("Outro", [
        "So, look in the middle, drop half, and repeat.",
        "Thanks for watching!",
    ]),
]
