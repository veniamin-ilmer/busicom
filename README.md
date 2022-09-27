# Busicom 141-PF GUI

You can see this gui live here - https://veniamin-ilmer.github.io/emu/busicom/

Building this GUI has been a completely different experience than building the chips supporting it.

A very important aspect is that web browsers have poor timing. The shortest amount of time they could sleep is about 10 milliseconds.

As a consequence, I am forced to break up the emulation into time chunks. Sleep for 10-20 milliseconds, then run several thousand CPU cycles. Then sleep again.

The emulated chips are completely unaware that they are run in this "run - sleep - run - sleep" mode.

With a refresh rate of 60 hz, that is 16 millseconds of sleep. The user is completely unaware of the chips in this "run - sleep - run - sleep" mode.

Since peripherals need to interact with the GUI and with the chips, I have broken up the peripheral cycles into two methods: `run_cycle` and `run_sleep_cycle`.

`run_cycle` methods run every cycle (Thousands per sleep). `run_sleep_cycle` methods run every refresh cycle. The slow gui stuff goes there.
