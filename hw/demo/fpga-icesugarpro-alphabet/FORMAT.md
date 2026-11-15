# The format for fpga-icesugarpro-alphabet

This is this package's layer of `hw/demo/FORMAT.md`, and it is read the way a
nested README is read: the parent states every rule that holds for any demo, this
file states what is true of this surface, and nothing here repeats the parent.

The parent leaves three things to a package. This file is those three.

| The parent leaves open | Here |
|---|---|
| the two events the cut is aligned on, and where they are in the footage | the two verdict lines, and their times |
| the vocabulary of the status line the console prints | four stages, five colours, two bits |
| the numbers of the last cut | the 20261115-102728 numbers |

What this apparatus is, and why it answers the question it answers, is in
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

The run uses stage 1 for both host acts and stage 3 for the verdict. The film it
is cut against carries one colour between its start and the red, so a script that
changed colour there would print a line the picture contradicts. A film of this
script, taken while it records, would let the two acts keep their own stages.

The iCELink offers three pins and this board drives two. The third, PA14 on F5,
is its SWD clock and never arrives, which is why the stage is two bits and there
are four stages rather than eight.

## The anchors

| | Line in the cast | In the footage |
|---|---|---|
| A | `[LED] red, steady` | 73.50 s |
| B | `[LED] green, steady` | 151.75 s |

Each line is printed immediately after the stage bits are written, so it is within
a tenth of a second of the LED changing. Neither line is the verdict: the verdict
is the pin, and `pin_match` on the line below states the digit.

The footage is the board filmed while the same stage sequence ran. That is why the
times are where they are, and why a new film needs its own two, read from the film
rather than copied from here. `cut.sh` passes them to the shared script, and
`FOOTAGE_A` and `FOOTAGE_B` override them for a new one. `../_shared/onsets.sh`
prints the colour of the LED over time so the two are measured, not guessed.

## The words a caption may not carry

The parent has a demo say what its own subject does. This subject is a format, so
`leaks.txt` forbids the vocabulary of a product that would consume it: a store, its
records, and the commercial engine that is built on this format. A caption that
reaches for one of those words is telling a different story.

## The programmer's conversation

`icesprog` is run in the foreground so its own lines reach the console: `gpio
mode`, `gpio write`, `gpio read ... return <n>`, and one `write 0x...` per 64 kB
while an image loads. Capturing it would put the whole minute in a buffer and
print a single line at the end, because a pipe makes the C library block-buffer.

## The cut of 20261115-102728

| | |
|---|---|
| Console anchors | 27.20 s, 106.81 s |
| Footage anchors | 73.50 s, 151.75 s |
| Trim, ratio | 46.767 s, 1.01745 |
| Anchors in the encoded file | 27.2 s, 106.8 s |
| Captions | 14 cues, last at 119.1 s |
| Output | 1920x1240, 129.6 s, 14.8 MiB, H.264 plus AAC mono |
| Host | `https://pub-06bcfdd98bd94801a748fa2ff0382400.r2.dev/demo/fpga-icesugarpro-alphabet/20261115-102728.mp4` |

The footage is 172.8 s and the take is 129.6 s, so the last 1.4 s of the take hold
the footage's final frame. That frame is the green verdict, which is steady, so
the still frame reads as the same light the run ended on.

The narration is the engine's female voice, `af_heart`, at 1.35 times its own
pace. The take was recorded while that voice spoke, so the track a viewer hears
is the one the room heard.
