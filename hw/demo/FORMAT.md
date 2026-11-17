# The demo video format

This is the convention for a demo video built from two recordings: a terminal
session that ran the software, and footage of the apparatus beside it. It applies
to every package under `hw/demo/` and to any later demo cut with
`hw/demo/_shared/final.sh`.

A finished artifact is one file, one frame, three layers: the console, the
apparatus, and the narration. Nothing else is drawn on it.

This document is the parent layer. It states every rule that holds for any
demo, and a package does not repeat them. Once a package has a cut it states its
own layer in a `FORMAT.md` beside its `narrate.sh`, read the way a nested README
is read: the parent holds the rules, the child holds only what is true of its
surface. `hw/demo/fpga-icesugarpro-alphabet/FORMAT.md` is the worked example.

## The frame

| Layer | Where | Size |
|---|---|---|
| Console | origin, 0,0 | 1920x1080 |
| Caption strip | directly under the console | 1920x160 |
| Apparatus footage | upper right, 920,40 | 960x540 |

The console is rendered from the cast at 131x31 with a 24 point font, which fills
1915x1075 of the 1080p frame; the remaining five pixels are the terminal's own
background, and `render.sh` pads with that same colour, so nothing is stretched.

The strip is added below the console in the same background, so the captions
belong to the terminal instead of floating over it. The finished frame is
1920x1240.

The footage sits in the upper right because a terminal scrolls: the newest lines
are at the bottom, and those are the ones a viewer reads an answer from. An inset
in the lower half covers exactly the lines that matter. The right and top margins
are both 40 pixels.

The cost of a half-frame inset is that a console line wider than about sixty-four
columns runs under it. That is accepted, and the numbers such a line carries also
appear in the page's table. A demo that needs those columns shrinks the inset
rather than moving it back down.

## The console

The console is evidence, so it carries only what a session carries.

| On the screen | Not on the screen |
|---|---|
| the command as it is typed, prefixed `$ ` | prose, narration, step headers |
| the tool's own stdout and stderr, verbatim | elapsed-second counters invented by the script |
| what the script parsed out of that output | a summary of output the script captured |
| one status line per state change, if the apparatus needs it | a caption of what that state means |

Two rules follow from this.

A command that runs for a minute has to show its own progress. A tool that
reports what it is doing is run in the foreground and never captured: capturing
it gives its stdout to a file, the C library block-buffers, and the whole minute
arrives as one line at the end.

A status line is allowed only for something the footage shows and the console
cannot. There is exactly one per state change, it is drawn in the state's own
colour behind `>>>`, and the package states the vocabulary in its own `FORMAT.md`. The
line exists so a viewer can compare the console with the apparatus at the same
second; it does not explain the state in words.

## The narration

The narration is written once per take and used twice: as the take's audio track,
and as captions. Both come from the same stamped log, so they cannot drift apart.

Narrate with `voice.sh`. It synthesises each stamped line, trims the silence
around it, mixes it at the time the run spoke it, normalises once to -16 LUFS,
and writes `demo.srt` from the same start and duration. A cue therefore covers
the words a viewer hears.

Placement uses the cast header's epoch. Every event time in an asciicast v2 is
relative to the `timestamp` in the header, and the stamped log is wall clock, so
the two share one origin. Do not anchor on the first echoed output: a console
that prints no prose has no echo to match.

Captions wrap at 42 characters a line, and the strip holds three lines. A line
that needs more is shortened, because a fourth line grows upward into the
console. A line is one thought. The text stays at the engine's level: a word from
the layer below it fails `check.sh`, because a demo is about what a record is,
not about how its address space is built.

Camera cues are spoken for the room and never stamped, so they cannot become a
caption about filming.

## The alignment

The console recording and the footage are separate takes of the same physical
sequence, aligned on two events and never on one.

One anchor is not enough. Whatever runs between the anchors takes a little longer
or shorter each time, so an offset that puts the first event exactly right leaves
the second one seconds off. Two anchors give an offset and a ratio:

```
out(t) = (source(t) - trim) * ratio
ratio  = (second - first) / (footage_b - footage_a)
trim   = footage_a - first / ratio
```

`setpts` multiplies timestamps, so the ratio is console seconds per footage
second. Invert it and the footage is stretched by the amount it should have been
compressed, which lands the second anchor late again, by about the amount the
first one was right.

The two anchors are the package's, and it names them in its own `FORMAT.md`: a line
the cast prints at each event, and the second in the footage at which the
apparatus shows it. `final.sh` takes them from `ANCHOR_A`, `ANCHOR_B`, `FOOTAGE_A` and
`FOOTAGE_B` and fails when one is missing, because a cut timed on a guessed
anchor is worse than no cut.

Measure the footage times, do not eyeball them. Sample a small crop where the
state changes and read the RGB mean as it changes, in steps small enough to see
the phase of anything that blinks:

```sh
ffmpeg -v error -ss <t> -i <file> -frames:v 1 \
    -vf "crop=<w>:<h>:<x>:<y>,scale=1:1" -f rawvideo -pix_fmt rgb24 - | od -An -tu1
```

The console times come from the cast, as the event time of the line the package
named. The trimmed and ratioed result belongs in the package's `FORMAT.md`.

## The encode

One `ffmpeg` call, one filter graph, in this order: pad, footage, then one
overlay per cue. Captions go last so nothing can cover them.

```
[0:v]pad=1920:1240:0:0:color=0x<bg>[base]
[1:v]trim=start=<trim>,setpts=(PTS-STARTPTS)*<ratio>,scale=960:540:flags=lanczos,setsar=1[board]
[base][board]overlay=920:40:eof_action=repeat[b0]
[b0][2:v]overlay=x=(W-w)/2:y=H-h-24:enable='between(t,<start>,<end>)'[b1]
... one per cue ...
[bN]null[v]
```

| Setting | Value | Why |
|---|---|---|
| Video | libx264, crf 20, yuv420p | text at 1080p stays legible |
| Frame rate | 30 | both sources are 30 |
| Audio | aac 128k mono from `voice.wav` | the take's own narration |
| Faststart | on | the player may seek before the file is complete |
| Footage end | `eof_action=repeat` | the footage is shorter than the run, and its last frame is the steady answer |

Captions are drawn as transparent PNGs and overlaid, not burned by the encoder.
The ffmpeg on this host carries no `subtitles`, `ass` or `drawtext` filter, and a
PNG keeps the position and the font under this document's control. The cue
images are built per take and deleted with the temporary directory.

The layout is the format's and stays in the script; the anchors come from the
package.

| Variable | Default | Meaning |
|---|---|---|
| `CAPTION_SIZE` | 30 | caption point size |
| `CAPTION_STRIP` | 160 | height of the strip under the console |
| `FONT_PATH` | Menlo | caption font |
| `BOARD_X`, `BOARD_Y` | 920, 40 | where the footage sits |

## Publishing

| | |
|---|---|
| Key | `demo/<package>/<take>[-vN].mp4` on the demo media bucket |
| Cache | `public, max-age=31536000, immutable` |
| Change | a new cut is a new key; a published key is never overwritten |

A recut of the same take takes a `-vN` suffix. The page's data file names the
current key, so an old link keeps playing what it played.

A page that autoplays is `autoplay muted`, because a browser refuses to start a
video with sound on its own. The file carries the narration regardless; the
player's own control unmutes it. Dropping `autoplay` to get sound means deciding
that the demo should wait for a click.

## Before a cut is published

| Check | How |
|---|---|
| the take is complete | `hw/demo/_shared/check.sh <take>` |
| both anchors land | sample the encoded file at the anchors, compare with the console lines, expect under 0.15 s |
| the console says what it should | read one frame at each anchor |
| the captions match the console | read one frame mid-sentence |
| the captions carry none of the package's forbidden words | the package's `leaks.txt`, read by `check.sh` |
| the audio is there and not silent | `ffprobe` for the stream, `volumedetect` for the level |
| the page names the new key | grep the built page for the key |

## What a package owns

| Thing | Where |
|---|---|
| the two anchors, and where they are in its footage | its `FORMAT.md`, passed to `final.sh` by `cut.sh` |
| the vocabulary of its status line | its `FORMAT.md` |
| the numbers of its last cut | its `FORMAT.md` |
| the lines a take must carry | `markers.txt` |
| the words a caption must not carry | `leaks.txt`, optional |
| the images the run programs, and the state the take opens on | `images.sh`, which builds them and stages the part |
| the run itself | `narrate.sh` |
