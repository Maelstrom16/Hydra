#pragma once

/**
 * Launches the Game Boy emulator using the given ROM.
 * 
 * @param file_path The path to the .gb or .gbc ROM file to be loaded.
 */
int gb_load_rom(char file_path[]);