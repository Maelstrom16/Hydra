#include "gbrom.h"
//--------------//

#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

#include "../common/hydraints.h"

static const int NINTENDO_LOGO_OFFSET = 0x0104;
static const u8 NINTENDO_LOGO[48] = {0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
                                     0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
                                     0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E};

bool gb_verify_rom_nintendo_logo(u8 *rom) {
    return memcmp(rom+NINTENDO_LOGO_OFFSET, NINTENDO_LOGO, 48ul);
}


static const int ROM_SIZE_OFFSET = 0x0148;

int gb_get_rom_size(u8 *rom) {
    switch(rom[ROM_SIZE_OFFSET]) {
        case 0x00: return 0x008000; // 32 KiB
        case 0x01: return 0x010000; // 64 KiB
        case 0x02: return 0x020000; // 128 KiB
        case 0x03: return 0x040000; // 256 KiB
        case 0x04: return 0x080000; // 512 KiB
        case 0x05: return 0x100000; // 1 MiB
        case 0x06: return 0x200000; // 2 MiB
        case 0x07: return 0x400000; // 4 MiB
        case 0x08: return 0x800000; // 8 MiB
        default: return 0x00;
    }
}

int gb_get_rom_bank_count(u8 *rom) {
    switch(rom[ROM_SIZE_OFFSET]) {
        case 0x00: return 2;
        case 0x01: return 4;
        case 0x02: return 8;
        case 0x03: return 16;
        case 0x04: return 32;
        case 0x05: return 64;
        case 0x06: return 128;
        case 0x07: return 256;
        case 0x08: return 512;
        default: return 0;
    }
}
