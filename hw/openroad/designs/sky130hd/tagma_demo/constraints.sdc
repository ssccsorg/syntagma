# Tagma demo top, SkyWater 130nm (sky130hd).
# 12 MHz board-equivalent clock: CLOCK_PERIOD = 83.33 ns.
create_clock -period 83.33 -name clk
set_input_delay -clock clk 1.0 [all_inputs]
set_output_delay -clock clk 1.0 [all_outputs]
