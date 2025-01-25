#pragma once

#include "../common/hydraints.h"
#include "gbenums.h"

struct gb_mbc {
    void (*read_mem)();
    void (*write_mem)();
};

struct gb_data {
    u8 *rom;
    u8 *memory;
    
    enum gb_revision revision;

    struct gb_mbc mbc;
};