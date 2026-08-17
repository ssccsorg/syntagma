# chton SRAM via OpenRAM

`chton_sram.py` configures OpenRAM to generate the physical SRAM for the chton segment store: a single-port, 16-bit wide macro holding the full Tagma addressable space (11,172 words, one Hangul code point per word).

## Status

Draft. Generation requires OpenRAM plus a PDK (SkyWater 130nm) and has not been run in CI yet. The 11,172 word count is not a power of two; see the fallback note in the configuration if the PDK flow rejects it.

## Run

```bash
git clone https://github.com/VLSIDA/OpenRAM.git
cd OpenRAM
pip install -e .
python -m openram --config chton_sram.py
```

OpenRAM emits GDSII, layout, timing and power models, and placement and routing views. Those outputs feed the OpenROAD standard cell flow in Phase 4.

## References

- OpenRAM: https://github.com/VLSIDA/OpenRAM
- SkyWater 130nm PDK: https://github.com/google/skywater-pdk
- OpenROAD: https://github.com/The-OpenROAD-Project/OpenROAD
