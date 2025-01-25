#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../common/hydraints.h"
#include "gbrom.h"

int gb_load_rom(char file_path[]) {
    // Check file extension
    char *ext = strrchr(file_path, '.');
    if (strcmp(ext, ".gb") == 0) {
        printf("Loading Game Boy ROM.\n");
    } else if (strcmp(ext, ".gbc") == 0) {
        printf("Loading Game Boy Color ROM.\n");
    } else {
        perror("Invalid file format. Ensure that the ROM has a .gb or .gbc extension.\n");
        return 1;
    }

    // Open test ROM
    FILE *_rom_file = fopen(file_path, "r");
    if (_rom_file == NULL) {
        perror("File read error. Please check the provided file path.\n");
        return 1;
    }

    // Copy ROM into memory, until end of header
    const int _START_TO_HEADER_LENGTH = 0x0150;
    u8 *ROM = malloc(_START_TO_HEADER_LENGTH);
    if (ROM == NULL) {
        perror("Insufficient memory to load ROM.\n");
        return 1;
    }
    fread(ROM, sizeof(u8), (unsigned long)_START_TO_HEADER_LENGTH, _rom_file);

    // Fetch ROM size from header and read remainder of ROM into memory
    int _rom_size = gb_get_rom_size(ROM);
    if (_rom_size == 0) {
        perror("Invalid ROM Size specified in cartridge header (0x0148). ROM is likely corrupt.\n");
        return 1;
    }
    ROM = realloc(ROM, _rom_size);
    if (ROM == NULL) {
        perror("Insufficient memory to load ROM.\n");
        return 1;
    }
    fread(ROM+_START_TO_HEADER_LENGTH, sizeof(u8), _rom_size-_START_TO_HEADER_LENGTH, _rom_file);

    // Close file pointer when done
    fclose(_rom_file);

    // Copy data of test ROM to memory buffer 
    u8 *MEM = malloc(0x10000);
    memcpy(MEM, ROM, 0x08000);

    
    // TODO: Launch CPU, graphics, and sound threads

    // Free resources when no longer needed
    free(ROM);
    free(MEM);

    return 0;
}