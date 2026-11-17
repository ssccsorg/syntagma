# The demos, packaged

A package is one story on one surface. Its name says which: the platform, the
board, and the topic. The video that carries it is built to one format, stated
in `FORMAT.md`: the frame, the console, the narration, the alignment with the
footage, the encode, and the checks before a cut is published. What a package
adds to that format is in its own `FORMAT.md`, which inherits the shared one the
way a nested README does.

| Package | Surface | What it argues |
|---|---|---|
| `fpga-icesugarpro-alphabet` | iCESugar-Pro, ECP5 | the alphabet itself: `tagma_core` writes one anchor per valid offset, the host sums them, and a die that shares no code with it decodes every value to the same number, refusing a set that is not the alphabet |
| `fpga-icesugarpro-exact` | iCESugar-Pro, ECP5 | the same anchors compared one line at a time, on two files that carry the same sum and disagree at two offsets, so the exchange a sum cannot see is refused by the die |

A package owns the script that is recorded (`narrate.sh`), the images built for
its run (`images.sh`), the steps a take has to carry (`markers.txt`), the words
a caption may not carry (`leaks.txt`), its takes (`out/`), and what its cut adds
to the shared format (`FORMAT.md` and `cut.sh`). Everything else lives in
`_shared/`, because the recorder, the render, the narration track, the cut, and
the check work on a take rather than on a story. A new surface is a new directory
with those things and a `demo.sh`, not a change to the recorder.

## Record it

```sh
hw/demo/fpga-icesugarpro-alphabet/demo.sh
```

That is the whole thing. The package's `demo.sh` hands its directory to
`_shared/run.sh`, which checks the board and the tools, runs the package's
`images.sh` if the images are to be built, puts the part in the state the take
opens on, records the run, renders the video, builds the narration track and its
captions, and checks the take before printing the paths under
`hw/demo/<package>/out/<timestamp>/`.

```sh
hw/demo/_shared/run.sh hw/demo/fpga-icesugarpro-alphabet --preflight   # check, record nothing
hw/demo/_shared/run.sh hw/demo/fpga-icesugarpro-alphabet --no-images   # record without rebuilding
```

The recording is the take. Start the camera on the first cue and stop on the
second. A cue is spoken for the room and never written into the captions, so a
take says nothing about filming.

### Film it beside the console

The console take and the film are two recordings of one run, so the film is made
while `demo.sh` runs. Put the phone in landscape at 1080p, start on the spoken
cue, and stop on the second one. The run stages the part before the recording
starts, which takes about two minutes, so the cue arrives late rather than at
once.

Keep the film beside the take, under `hw/demo/<package>/out/footage/`, which is
ignored by git.

Then measure the two verdicts, because they are what the cut is timed by. Read
the box the LED sits in from one still, then let `onsets.sh` report the colour
over time:

```sh
ffmpeg -ss 30 -i film.mov -frames:v 1 /tmp/frame.png
hw/demo/_shared/onsets.sh film.mov <x>:<y>:<w>:<h> 0 200 0.5
```

Each line is the mean colour of that box and a marker where the colour changed.
Red and green are steady, so the first sample of each is the onset. Pass those
two to the cut:

```sh
FOOTAGE_A=<red> FOOTAGE_B=<green> hw/demo/<package>/cut.sh <take> <film>
```

## What it needs

| | |
|---|---|
| The recorder | `asciinema`, and `agg` for the render |
| The cut | `ffmpeg`, `ffprobe`, and ImageMagick |
| The voice | `voice`, the Kokoro TTS on Apple Silicon; `say` is the fallback |

What the surface itself needs is the package's: `fpga-icesugarpro-alphabet` needs
the board on USB, `icesprog`, and the FPGA toolchain that OSS CAD Suite carries,
and it states all of that in its own `README.md`.

## The files

| | |
|---|---|
| `<package>/demo.sh` | the one line, which hands the package to the recorder |
| `<package>/narrate.sh` | the run itself, which is what the cast contains |
| `<package>/images.sh` | the images that run programs, and the part staged for the take |
| `<package>/markers.txt` | the steps a take of that package has to carry |
| `<package>/leaks.txt` | words a caption may not carry, because they name another story |
| `<package>/FORMAT.md` | what this apparatus adds to the format: anchors, status vocabulary, cut numbers |
| `<package>/cut.sh` | the cut, with those anchors handed to the shared script |
| `<package>/out/` | the takes and the images; ignored by git |
| `_shared/run.sh` | the preflight, the recording, and the calls that follow it |
| `_shared/check.sh` | whether a take is fit to show, run last by `run.sh` |
| `_shared/render.sh` | the cast to a 1920x1080 video |
| `_shared/voice.sh` | the narration track and the captions, from the stamped log |
| `_shared/final.sh` | the published cut: the console, the footage timed to it, and the captions |
| `_shared/onsets.sh` | the LED's colour over time in a film, so a cut's anchors are measured |
| `FORMAT.md` | the parent layer, which every package's `FORMAT.md` inherits |

## The narration track and the captions

`voice.sh` writes two files from the same timing: `voice.wav`, the narration on
the video's own timeline (mono, 44.1 kHz, 16-bit, as long as the video), and
`demo.srt`, the same lines as captions. The published cut carries both, so a
viewer can listen to the narration or read it, and the console behind it stays a
session.

Captions are SubRip because every editor imports it and the text stays plain and
editable. A cue starts when its line was spoken and ends when that line stops
sounding: both numbers come from the synthesis, so the captions cannot drift from
the voice. The leading and trailing silence of a synthesised line is trimmed
first, so a cue covers the words rather than the file, and the gap to the next cue
is the silence a viewer actually hears. Text wraps at 42 characters a line, which
reads at 1080p without shrinking.

`_shared/final.sh` draws the cues over the strip rather than asking the encoder to
burn a subtitle file, because the ffmpeg on this host carries no subtitle filter.

```sh
hw/demo/_shared/voice.sh <take>

# the published cut, with this package's anchors
hw/demo/fpga-icesugarpro-alphabet/cut.sh <take> <footage>
```

Each line is synthesised once and mixed at the time the run spoke it. That also
means a cut can carry a different voice from the one the room heard: a cue starts
at the stamped time, so changing the voice moves the end of a line and no start.
`narrate.sh` stamps every line into `voice.log` as it speaks, and the cast's
header carries the epoch its clock starts from, so the two share one origin
without matching an echoed line. The mix is normalised once to -16 LUFS, because
the engines do not agree on level and the track should not depend on which one
spoke it.

| Variable | Default | Meaning |
|---|---|---|
| `TTS` | `voice` | `voice` for Kokoro on Apple Silicon, `say` for the system voice, `none` for a silent run |
| `TTS_VOICE` | `af_heart` | Kokoro voice name; `af_heart` is its default female voice, `am_michael` the male one |
| `TTS_SPEED` | `1.35` | Kokoro speed factor; 1.0 is the model's own pace and 1.35 matches the old system voice |
| `SAY_RATE` | `178` | words a minute, used only by `say` |

## What the console shows

The run prints the commands it issues and the answers the tools give, as they
come. Nothing is captured and summarised, so the console shows a session rather
than a report of one. A tool that reports its progress reports it on the screen:
in this package the programmer names every 64 kB it lands, and each GPIO call
prints the two lines it returns. A command goes to the terminal and not into the
value the run captures, so a read shows its conversation and still returns one
digit.

The narration is not printed. It is the caption under the console, and the only
lines that are neither a command nor a tool's own output are a package's status
lines: `>>> [LED] red, steady` in this package, which its `FORMAT.md` explains.

## The take is checked before anyone sees it

A take is one console recording of the whole run, so a take that stops after its
first line is a broken take rather than a short demo. `run.sh` ends by calling
`check.sh` on what it just recorded, and a take that fails the check exits
non-zero instead of being handed over.

The check reads the marker file the take carries, so each package states its own
steps, and requires every one in the cast, in order, a cast long enough for a full
run, a rendered video whose length matches the cast, a narration track as long as
that video, captions whose last cue stays inside it, no two cues closer than six
tenths of a second (a line read straight into the next one is what a viewer hears
as rushed), and no caption that carries a word from the package's `leaks.txt`. The
same file works by hand on any take.

```sh
hw/demo/_shared/check.sh hw/demo/fpga-icesugarpro-alphabet/out/<take>
```

## The surface

What the apparatus is for, what it proves, and what it does not claim belong to
the package, along with what it needs from the bench.
`fpga-icesugarpro-alphabet` states them in its own `README.md`: what the die
decodes, what the host sums, and what none of it claims.

## The frame

The console is delivered as 1920x1080. The terminal renders at 1915x1075 with the
default font and the last five pixels of width and height are the terminal's own
background, so nothing is stretched.

`_shared/final.sh` builds the published frame instead, and `FORMAT.md` states it:
the console at 1920x1080, a 160-pixel strip under it in the terminal's own
background for the captions, and the footage at half the frame in the upper right,
where it does not cover the newest console lines. The take's `voice.wav` is muxed
in as the audio track, so the picture and the narration come from one cut. The two
anchors the footage is timed by are the package's, and its `FORMAT.md` states
them.

`agg` writes a GIF and nothing else, so the video is one encode away from it. The
encode resamples at 30 frames a second instead of copying the GIF's per-frame
delays, because a player may clamp those delays and a run that should last four
minutes comes out at five seconds. Resampling turns each pause into the frames
that fill it. The GIF is kept beside the video as a preview.
