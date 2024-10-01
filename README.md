# Plan

split keyboard, fun to make and fun to have

## Requirements

keys + key caps
===============
- 60%, 61 keys
- mechanical keys
  - pros: maybe sounds good
  - cons: misclick frequently, more tiring to type, slow
- flat keys
  - pros: easier to press, faster to type
  - cons: less cool?
- *decision*
  - search for flat keys and mechanical keys, should be able to make keybaords with either in the same way

microcontroller
===============
- connection type
  - wired
    - pros: no battery, simpler, would be fun to program bluetooth controller
    - cons: annoying while using
  - bluetooth:
    - pros: clean to use
    - cons: battery is annoying
  - *decision*
    - wired to start - bluetooth will make the project more complicated than necessary
- possible options
  - ATmega32U4 (widely used, looks small)
    - as raw IC mounted on the PCB (more complicated, so don't do) or as a "hat" on a breakout board
      - https://www.microchip.com/en-us/product/atmega32u4
  - STM32F103 (powerful, bigger)
- *variables to minimize*
  - size
  - power consumption
  - key-press latency

eece / computer
===============
- need to be able to detect any combination of keys together simultaneously
- sw
  - rust ;)
  - implements USB HID 1.11 + reading gpio inputs
- elec
  - nice looking pcb
  - reading keys
    - keyboard matrix
      - pros: simple
    - shift registers (74HC165) to multiplex data in
      - pros: sort of more interesting
    - *decision*
      - keyboard matrix. simpler is better.
  - is there a little led grid that I could have sitting somewhere flashing when keys are pressed?

## Details

keyboard matrix
===============

- layout?
