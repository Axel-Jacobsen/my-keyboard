# Plan

split keyboard, fun to make and fun to have

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
- microcontroller -> pcb - how to connect?
  - solder directly to pcb
    - pros: fuckin sick, may be cheaper? maybe not
    - cons: much more difficult to manufacture
  - get breakout and solder the pins
    - pros: much easier to deal with
    - cons: less fkn tight
  - *decision*
    - do breakout for v1
- *variables to minimize*
  - size
  - power consumption
  - key-press latency

eece / computer
===============
- need to be able to detect any combination of keys together simultaneously
- sw
  - rust ;)
    - avr rust book: https://book.avr-rust.org/
    - https://docs.rs/avrd/latest/avrd/atmega16u4/index.html
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

keys + key caps
===============

- A 61 key keyboard is common - a "60%" - but what do I need?
  - want 60 keys minimum
    - 26 for the letters of the alphabet
    - 10 for decimal digits
    - 1 for +
    - 1 for -
    - 1 for spacebar
    - 3 for control, option, command
    - 1 for escape (in caps lock position)
    - 1 for tilde
    - 1 for tab
    - 1 for shift
    - 1 for ;
    - 1 for '
    - 1 for delete
    - 2 for < and >
    - 2 for []
    - 1 for ?
    - 1 for \
    - 4 for arrow keys
    - 1 for special key 1
  - with antoher 5,
    - 1 for split spacebar
    - 1 for another special key
- key type
  - mechanical keys
    - pros: maybe sounds good
    - cons: misclick frequently, more tiring to type, slow
  - flat keys
    - pros: easier to press, faster to type
    - cons: less cool?
  - *decision*
    - search for flat keys and mechanical keys, should be able to make keybaords with either in the same way
