#include <stdio.h>

#include "gameboy/gbmain.h"

#include "common/readwrite.h"

int main() {
    u8 test[64] = {1, 2, 3, 4, 0x35, 0xC9, 7, 8, 9};
    printf("%x\n", read_u8(test, 4));
    printf("%x\n", read_u16(test, 4, LITTLE_END));
    printf("%x\n", read_u16(test, 4, BIG_END));
    //gb_load_rom("testrom.gb");
    return 0;
}