#pragma once

#include <stdbool.h>

#include "../common/hydraints.h"

/**
 * Checks whether the Nintendo logo in the cartridge header
 * is formatted properly.
 * 
 * @param rom A pointer to the ROM.
 * @return True if the logo is formatted properly, false otherwise.
 */
bool gb_verify_rom_nintendo_logo(u8 *rom);

/**
 * Returns the amount of data in bytes the given ROM contains
 * based on the value presented in the cartridge header.
 * 
 * @param rom A pointer to the ROM.
 * @return The number of bytes within the ROM.
 */
int gb_get_rom_size(u8 *rom);

/**
 * Returns the number of ROM banks the given ROM contains
 * based on the value presented in the cartridge header.
 * 
 * @param rom A pointer to the ROM.
 * @return The number of ROM banks within the ROM.
 */
int gb_get_rom_bank_count(u8 *rom);

/**
 * Checks whether the header checksum is valid.
 * 
 * @param rom A pointer to the ROM.
 * @return True if the header checksum is valid, false otherwise.
 */
bool gb_verify_rom_header_checksum(u8 *rom);

/**
 * Checks whether the global checksum is valid.
 * 
 * @param rom A pointer to the ROM.
 * @return True if the global checksum is valid, false otherwise.
 */
bool gb_verify_rom_global_checksum(u8 *rom);