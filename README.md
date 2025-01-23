# Hydra
An emulator for a number of retro game systems, written mostly to play around with C.

## Files
* **src/** - *Contains all C code.*
    * **common/** - *Contains files to be reused between multiple emulators.*
        * **hydraints.h** - *Contains simplified typedefs for number types.*
    * **gameboy/** - *Contains files to be used by the Game Boy emulator.*
        * **gbmain.c/h** - *Contains the code necessary to initialize and launch the Game Boy emulator.*
        * **gbrom.c/h** - *Contains helper functions for analyzing Game Boy ROM files.*
    * **main.c** - *Contains the main method. Currently only used for testing.*
* **Makefile** - *Contains simple build instructions for compilation with clang.*
* **README.md** - *This file*