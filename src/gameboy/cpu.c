#include "../common/hydraints.h"
#include "gbenums.h"
#include "gbdata.h"

union _reg_pair{
    u16 r16;
    u8 r8[2];
};

struct gb_cpu_registers {
    union _reg_pair _af;
    union _reg_pair _bc;
    union _reg_pair _de;
    union _reg_pair _hl;
    u16 _pc;
    u16 _sp;
    u8 _ime;
};

#define AF _af.r16
#define A _af.r8[1]
#define F _af.r8[0]

#define BC _bc.r16
#define B _bc.r8[1]
#define C _bc.r8[0]

#define DE _de.r16
#define D _de.r8[1]
#define E _de.r8[0]

#define HL _hl.r16
#define H _hl.r8[1]
#define L _hl.r8[0]

#define PC _pc
#define SP _sp
#define IME _ime

static void gb_set_register_defaults(struct gb_cpu_registers *registers, u8 *memory, enum gb_revision revision) {
    // TODO: Replace DMG0 register values with values specific to the provided revision
    registers->A = 0x01;
    registers->F = 0b0000;
    registers->B = 0xFF;
    registers->C = 0x13;
    registers->D = 0x00;
    registers->E = 0xC1;
    registers->H = 0x84;
    registers->L = 0x03;
    registers->PC = 0x0100;
    registers->SP = 0xFFFE;
    registers->IME = 0;
}

int gb_cpu_thread(struct gb_data *data) {
    // Initialize CPU/Hardware registers.
    struct gb_cpu_registers registers;
    gb_set_register_defaults(&registers, data->memory, data->revision);
    
    return 0;
}