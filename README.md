# whiskers
Rust driver and util for RF devices (alpha-quality, feature-incomplete)

Mostly there are two binaries, `whiskers` and `whiskers-bl` for regular mode and CC-bootloader mode respectively.

These are tested with a YARD Stick One and a PandwaRF.  Currently, only supports UART-over-USB, but eventually this should run fine over SPI

# Quick HOWTO

Importantly, you must select the devices that are targeted, it does not choose for you.  To run on all USB devices, pass `--usb-all`, otherwise select specific addresses using `--usb-addr bus,device`

It will default to matching known RFCat USB vendor and products, but that may be overridden in the command line, which can effectively narrow to specific devices (see bootloader example), or expand to currently-unknown devices (YARD Stick Two?).

# Things whiskers can do (subcommands)

# **has-bootloader**: tests for CC-Bootloader

(yes, I copypasted `ping`, sue me)

```
# false on a PandwaRF
$ whiskers has-bootloader --usb-all --usb-vp 1d50,60ff
RFCat: b001 d092 v1d50 p60ff
  false (396 us)
```

```
# true on a YS1
$ whiskers has-bootloader --usb-all --usb-vp 1d50,605b
RFCat: b001 d084 v1d50 p605b
  true (291 us)
```

# **bootloader**: place rfcats in bootloader mode

(non-working)

# **buildname**: display CC firmware's build name

```
$ whiskers buildname --usb-all
RFCat: b001 d092d v1d50 p60ff
  buildname: GollumRfBigCCtlRevD
RFCat: b001 d084d v1d50 p605b
  buildname: YARDSTICKONE r0543
```

# **compiler**: display CC firmware's complier (if present)

```
$ whiskers compiler --usb-all
RFCat: b001 d092d v1d50 p60ff
  no-compiler
RFCat: b001 d084d v1d50 p605b
  compiler: SDCCv370
```

# **ping**: simple reflected command
```
$ whiskers ping --usb-all
RFCat: b001 d092 v1d50 p60ff
  true (297 us)
RFCat: b001 d084 v1d50 p605b
  true (2840 us)
```
