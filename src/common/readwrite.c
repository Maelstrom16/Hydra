#include "hydraints.h"
#include "hydraenums.h"

u8 read_u8(u8 *buffer, u64 address) {
    return buffer[address];
}

u16 read_u16(u8 *buffer, u64 address, enum endianness endian) {
    u16 result = 0;
    const int MAX_BIT_INDEX = 1; // Because there are 2 bits total. (2 - 1 = 1)
    for (int i = 0; i <= MAX_BIT_INDEX; i++) {
        result |= (buffer[address + (endian == LITTLE_END ? i : (MAX_BIT_INDEX-i))] << (i * 8));
    }
    return result;
}