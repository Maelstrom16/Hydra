#pragma once

#include "hydraints.h"
#include "hydraenums.h"

u8 read_u8(u8 *buffer, u64 address);

u16 read_u16(u8 *buffer, u64 address, enum endianness);