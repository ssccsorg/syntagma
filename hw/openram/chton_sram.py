# OpenRAM configuration for the chton segment store.
#
# chton maps the Tagma coordinate space (11,172 syllables) onto physical
# SRAM. This configuration generates a single-port, 16-bit SRAM macro
# addressed by the 14-bit offset derived in hw/rtl/tagma_decoder.v.
#
# Usage:
#   python -m openram --config chton_sram.py
#
# Requires OpenRAM (https://github.com/VLSIDA/OpenRAM) and a PDK
# (e.g. SkyWater 130nm, https://github.com/google/skywater-pdk).

word_size = 16          # one Hangul code point per word
num_words = 11172       # full Tagma space: 0xAC00..0xD7A3, 19 * 21 * 28
num_rw_ports = 1        # single read-write port
num_r_ports = 0
num_w_ports = 0
tech_name = "sky130"    # SkyWater 130nm
nominal_corner_only = True
process_corners = ["SS", "TT", "FF"]
route_supplies = True
check_lvsdrc = True

# Note: 11172 is not a power of two. If the PDK flow rejects non-power-of-
# two word counts, fall back to num_words = 16384 (2 ** 14) and treat the
# extra 5,212 entries as reserved. The 14-bit address space covers the
# full valid range either way.
