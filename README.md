# aer
Aer - ePaper-Thermometer 

Choose your features before running:
- (optional) 'simulator' for simulating a display
- 'epd2in9' or 'epd4in2' for the displays

# Packaging

To manually package e.g. for a Pi Zero W + 4.2" Waveshare e-Paper module, run the following:

```
cargo deb --variant epd4in2 --target arm-unknown-linux-gnueabihf
```
