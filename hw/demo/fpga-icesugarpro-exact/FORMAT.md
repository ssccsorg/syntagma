# The format for fpga-icesugarpro-exact

This is this package's layer of `hw/demo/FORMAT.md`, and it is read the way a
nested README is read: the parent states every rule that holds for any demo, this
file states what is true of this surface, and nothing here repeats the parent.

The parent leaves three things to a package. This file is those three.

| The parent leaves open | Here |
|---|---|
| the two events the cut is aligned on, and where they are in the footage | the two verdict lines, and their times |
| the vocabulary of the status line the console prints | four stages, five colours, two bits |
| the numbers of the last cut | the numbers below |

What this apparatus is, and why it answers a question a sum cannot, is in
`README.md` beside this file. The line that records a take is `demo.sh`, and
`cut.sh` hands these anchors to the shared cut.

## The status vocabulary

The console prints one line per stage change, drawn in that colour behind `>>>`:

```
>>> [LED] <colour>, <steady or blink>
```

| Stage | Bits PA1, PA0 | Colour | ANSI | Line |
|---|---|---|---|---|
| 0 | 0, 0 | blue | 94 | `[LED] blue, blink` |
| 1 | 1, 0 | cyan | 96 | `[LED] cyan, blink` |
| 2 | 0, 1 | yellow | 93 | `[LED] yellow, blink` |
| 3 | 1, 1 | red | 91 | `[LED] red, steady` |
| 3 | 1, 1 | green | 92 | `[LED] green, steady` |

Stage 3 is one stage with two colours: which one appears is the die's answer, not
the host's. Every other stage blinks at about a second and the verdict is steady,
because a light that sits still reads as a photograph of a part.

The run uses stages 0 and 1 for the two host acts, stage 3 for the verdict, and
stage 2 while the image is written. The film it is cut against carries the same
four, so every LED line in the take can be checked against the picture.

The iCELink offers three pins and this board drives two. The third, PA14 on F5,
is its SWD clock and never arrives, which is why the stage is two bits and there
are four stages rather than eight.

## The anchors

| | Line in the cast | In the footage |
|---|---|---|
| A | `[LED] red, steady` | 35.60 s |
| B | `[LED] green, steady` | 123.70 s |

Each line is printed immediately after the stage bits are written, so it is within
a tenth of a second of the LED changing. Neither line is the verdict: the verdict
is the pin, and `pin_match` on the line below states the digit.

The footage is the board filmed while this stage sequence ran. These two times are the
ones measured in the film this take was shot with, and the film is named in the cut
section below. A new film needs its own two, read from the film rather than copied from
here. `cut.sh` passes them to the shared script, and `FOOTAGE_A` and `FOOTAGE_B` override
them for a new one. `../_shared/onsets.sh` prints the colour of the LED over time so the
two are measured, not guessed.

## The words a caption may not carry

The parent has a demo say what its own subject does. This subject is a comparison
a format makes possible, so `leaks.txt` forbids the vocabulary of a product that
would consume the format: a store, its records, and the commercial engine built on
this format. A caption that reaches for one of those words is telling a different
story.

## The programmer's conversation

`icesprog` is run in the foreground so its own lines reach the console: `gpio
mode`, `gpio write`, `gpio read ... return <n>`, and one `write 0x...` per 64 kB
while an image loads. Capturing it would put the whole minute in a buffer and
print a single line at the end, because a pipe makes the C library block-buffer.

## The cut of 20261115-114545

| | |
|---|---|
| Console anchors | 39.70 s, 127.87 s |
| Footage anchors | 35.60 s, 123.70 s |
| Pad, trim | 4.064 s, 0.000 s |
| Ratio | 1.00084 |
| Anchors in the encoded file | 39.7 s, 127.9 s |
| Captions | 15 cues, last at 141.3 s |
| Output | 1920x1240, 154.5 s, 25.2 MiB, H.264 plus AAC mono |
| Footage | `out/board.mov`, 1284x722, 150.3 s, rotation -180 |
| Host | `https://pub-06bcfdd98bd94801a748fa2ff0382400.r2.dev/demo/fpga-icesugarpro-exact/20261115-114545.mp4` |

This film was shot with this take rather than borrowed from the alphabet package, so
the two clocks differ only by the 4.064 s the camera started early, and the ratio is
1.00084 rather than the 1.001 the alphabet cut reports. The film carries blue to
23.5 s, cyan to 35.6 s, red to 46.5 s, yellow while the part is written, and green from
123.7 s, and every one of those lands within half a second of the console line that
states it.

The take's console runs to 151.8 s and the film to 150.3 s, so the encode runs to
154.5 s and its last 2.7 s hold the console's final frame.

The narration is the engine's female voice, `af_heart`, at 1.35 times its pace. The track
is synthesised from the log the take stamped, so a cut can carry a different voice
without moving a cue.
