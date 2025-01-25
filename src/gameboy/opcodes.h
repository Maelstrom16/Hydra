#include <stdlib.h>
#include <stdbool.h>

#include "../common/hydraints.h"
#include "../common/readwrite.h"

// OPCODE FUNCTIONS //
// return the number of cycles taken for the opcode to complete.

// 8-bit Loads
int LD_r_r(u8 *r1, u8 *r2) {
    return 0; // STUB
}

int LD_r_n(u8 *r) {
    return 0; // STUB
}

int LD_r_irr(u8 *r) {
    return 0; // STUB
}

int LD_irr_r(u8 *r) {
    return 0; // STUB
}

int LD_irr_n(u8 *r) {
    return 0; // STUB
}

int LD_r_inn(u8 *r) {
    return 0; // STUB
}

int LD_inn_r(u8 *r) {
    return 0; // STUB
}

int LDH_r_ir(u8 *r) {
    return 0; // STUB
}

int LDH_ir_r(u8 *r) {
    return 0; // STUB
}

int LDH_r_in(u8 *r) {
    return 0; // STUB
}

int LDH_in_r(u8 *r) {
    return 0; // STUB
}

int LD_r_ird(u8 *r) {
    return 0; // STUB
}

int LD_ird_r(u8 *r) {
    return 0; // STUB
}

int LD_r_iri(u8 *r) {
    return 0; // STUB
}

int LD_iri_r(u8 *r) {
    return 0; // STUB
}

// 16-bit Loads
int LD_rr_nn(u8 *r) {
    return 0; // STUB
}

int LD_inn_rr(u8 *r) {
    return 0; // STUB
}

int LD_rr_rr(u8 *r) {
    return 0; // STUB
}

int PUSH_rr(u8 *r) {
    return 0; // STUB
}

int POP_rr(u8 *r) {
    return 0; // STUB
}

int LD_rr_rre(u8 *r) {
    return 0; // STUB
}

// 8-bit Arithmetic
int ADD_r(u8 *r) {
    return 0; // STUB
}

int ADD_irr(u8 *r) {
    return 0; // STUB
}

int ADD_n(u8 *r) {
    return 0; // STUB
}

int ADC_r(u8 *r) {
    return 0; // STUB
}

int ADC_irr(u8 *r) {
    return 0; // STUB
}

int ADC_n(u8 *r) {
    return 0; // STUB
}

int SUB_r(u8 *r) {
    return 0; // STUB
}

int SUB_irr(u8 *r) {
    return 0; // STUB
}

int SUB_n(u8 *r) {
    return 0; // STUB
}

int SBC_r(u8 *r) {
    return 0; // STUB
}

int SBC_irr(u8 *r) {
    return 0; // STUB
}

int SBC_n(u8 *r) {
    return 0; // STUB
}

int CP_r(u8 *r) {
    return 0; // STUB
}

int CP_irr(u8 *r) {
    return 0; // STUB
}

int CP_n(u8 *r) {
    return 0; // STUB
}

int INC_r(u8 *r) {
    return 0; // STUB
}

int INC_irr(u8 *r) {
    return 0; // STUB
}

int DEC_r(u8 *r) {
    return 0; // STUB
}

int DEC_irr(u8 *r) {
    return 0; // STUB
}

int AND_r(u8 *r) {
    return 0; // STUB
}

int AND_irr(u8 *r) {
    return 0; // STUB
}

int AND_n(u8 *r) {
    return 0; // STUB
}

int OR_r(u8 *r) {
    return 0; // STUB
}

int OR_irr(u8 *r) {
    return 0; // STUB
}

int OR_n(u8 *r) {
    return 0; // STUB
}

int XOR_r(u8 *r) {
    return 0; // STUB
}

int XOR_irr(u8 *r) {
    return 0; // STUB
}

int XOR_n(u8 *r) {
    return 0; // STUB
}

int CCF() {
    return 0; // STUB
}

int SCF() {
    return 0; // STUB
}

int DAA() {
    return 0; // STUB
}

int CPL() {
    return 0; // STUB
}

// 16-bit Arithmetic
int INC_rr(u8 *r) {
    return 0; // STUB
}

int DEC_rr(u8 *r) {
    return 0; // STUB
}

int ADD_rr_rr(u8 *r) {
    return 0; // STUB
}

int ADD_rr_e(u8 *r) {
    return 0; // STUB
}

// Bit Operations
int RLC_r(u8 *r) {
    return 0; // STUB
}

int RLC_irr(u8 *r) {
    return 0; // STUB
}

int RRC_r(u8 *r) {
    return 0; // STUB
}

int RRC_irr(u8 *r) {
    return 0; // STUB
}

int RL_r(u8 *r) {
    return 0; // STUB
}

int RL_irr(u8 *r) {
    return 0; // STUB
}

int RR_r(u8 *r) {
    return 0; // STUB
}

int RR_irr(u8 *r) {
    return 0; // STUB
}

int SLA_r(u8 *r) {
    return 0; // STUB
}

int SLA_irr(u8 *r) {
    return 0; // STUB
}

int SRA_r(u8 *r) {
    return 0; // STUB
}

int SRA_irr(u8 *r) {
    return 0; // STUB
}

int SWAP_r(u8 *r) {
    return 0; // STUB
}

int SWAP_irr(u8 *r) {
    return 0; // STUB
}

int SRL_r(u8 *r) {
    return 0; // STUB
}

int SRL_irr(u8 *r) {
    return 0; // STUB
}

int BIT_b_r(u8 *r) {
    return 0; // STUB
}

int BIT_b_irr(u8 *r) {
    return 0; // STUB
}

int RES_b_r(u8 *r) {
    return 0; // STUB
}

int RES_b_irr(u8 *r) {
    return 0; // STUB
}

int SET_b_r(u8 *r) {
    return 0; // STUB
}

int SET_b_irr(u8 *r) {
    return 0; // STUB
}


// Control Flow

// Misc. Instructions
int NOP() {
    // Do nothing
    return 0;
}

int STOP() {
    return 0; // STUB
}

int HALT() {
    return 0; // STUB
}

int EI() {
    return 0; // STUB
}

int DI() {
    return 0; // STUB
}

int gb_run_opcode(u8 *pc, struct gb_data *data) {
    switch(opcode) {
        case 0x00: return NOP();
        case 0x01: 
        case 0x02:
        case 0x03:
        case 0x04:
        case 0x05:
        case 0x06:
        case 0x07:
        case 0x08:
        case 0x09:
        case 0x0A:
        case 0x0B:
        case 0x0C:
        case 0x0D:
        case 0x0E:
        case 0x0F:

        case 0x10:
        case 0x11:
        case 0x12:
        case 0x13:
        case 0x14:
        case 0x15:
        case 0x16:
        case 0x17:
        case 0x18:
        case 0x19:
        case 0x1A:
        case 0x1B:
        case 0x1C:
        case 0x1D:
        case 0x1E:
        case 0x1F:

        case 0x20:
        case 0x21:
        case 0x22:
        case 0x23:
        case 0x24:
        case 0x25:
        case 0x26:
        case 0x27:
        case 0x28:
        case 0x29:
        case 0x2A:
        case 0x2B:
        case 0x2C:
        case 0x2D:
        case 0x2E:
        case 0x2F:

        case 0x30:
        case 0x31:
        case 0x32:
        case 0x33:
        case 0x34:
        case 0x35:
        case 0x36:
        case 0x37:
        case 0x38:
        case 0x39:
        case 0x3A:
        case 0x3B:
        case 0x3C:
        case 0x3D:
        case 0x3E:
        case 0x3F:

        case 0x40: return LD_r_r(B, B);
        case 0x41: return LD_r_r(B, C);
        case 0x42: return LD_r_r(B, D);
        case 0x43: return LD_r_r(B, E);
        case 0x44: return LD_r_r(B, H);
        case 0x45: return LD_r_r(B, L);
        case 0x46: return LD_r_irr(B, HL);
        case 0x47: return LD_r_r(B, A);
        case 0x48: return LD_r_r(C, B);
        case 0x49: return LD_r_r(C, C);
        case 0x4A: return LD_r_r(C, D);
        case 0x4B: return LD_r_r(C, E);
        case 0x4C: return LD_r_r(C, H);
        case 0x4D: return LD_r_r(C, L);
        case 0x4E: return LD_r_irr(C, HL);
        case 0x4F: return LD_r_r(C, A);

        case 0x50: return LD_r_r(D, B);
        case 0x51: return LD_r_r(D, C);
        case 0x52: return LD_r_r(D, D);
        case 0x53: return LD_r_r(D, E);
        case 0x54: return LD_r_r(D, H);
        case 0x55: return LD_r_r(D, L);
        case 0x56: return LD_r_irr(D, HL);
        case 0x57: return LD_r_r(D, A);
        case 0x58: return LD_r_r(E, B);
        case 0x59: return LD_r_r(E, C);
        case 0x5A: return LD_r_r(E, D);
        case 0x5B: return LD_r_r(E, E);
        case 0x5C: return LD_r_r(E, H);
        case 0x5D: return LD_r_r(E, L);
        case 0x5E: return LD_r_irr(E, HL);
        case 0x5F: return LD_r_r(E, A);

        case 0x60:
        case 0x61:
        case 0x62:
        case 0x63:
        case 0x64:
        case 0x65:
        case 0x66:
        case 0x67:
        case 0x68:
        case 0x69:
        case 0x6A:
        case 0x6B:
        case 0x6C:
        case 0x6D:
        case 0x6E:
        case 0x6F:

        case 0x70:
        case 0x71:
        case 0x72:
        case 0x73:
        case 0x74:
        case 0x75:
        case 0x76:
        case 0x77:
        case 0x78:
        case 0x79:
        case 0x7A:
        case 0x7B:
        case 0x7C:
        case 0x7D:
        case 0x7E:
        case 0x7F:

        case 0x80:
        case 0x81:
        case 0x82:
        case 0x83:
        case 0x84:
        case 0x85:
        case 0x86:
        case 0x87:
        case 0x88:
        case 0x89:
        case 0x8A:
        case 0x8B:
        case 0x8C:
        case 0x8D:
        case 0x8E:
        case 0x8F:

        case 0x90:
        case 0x91:
        case 0x92:
        case 0x93:
        case 0x94:
        case 0x95:
        case 0x96:
        case 0x97:
        case 0x98:
        case 0x99:
        case 0x9A:
        case 0x9B:
        case 0x9C:
        case 0x9D:
        case 0x9E:
        case 0x9F:

        case 0xA0:
        case 0xA1:
        case 0xA2:
        case 0xA3:
        case 0xA4:
        case 0xA5:
        case 0xA6:
        case 0xA7:
        case 0xA8:
        case 0xA9:
        case 0xAA:
        case 0xAB:
        case 0xAC:
        case 0xAD:
        case 0xAE:
        case 0xAF:

        case 0xB0:
        case 0xB1:
        case 0xB2:
        case 0xB3:
        case 0xB4:
        case 0xB5:
        case 0xB6:
        case 0xB7:
        case 0xB8:
        case 0xB9:
        case 0xBA:
        case 0xBB:
        case 0xBC:
        case 0xBD:
        case 0xBE:
        case 0xBF:

        case 0xC0:
        case 0xC1:
        case 0xC2:
        case 0xC3:
        case 0xC4:
        case 0xC5:
        case 0xC6:
        case 0xC7:
        case 0xC8:
        case 0xC9:
        case 0xCA:
        case 0xCB:
        case 0xCC:
        case 0xCD:
        case 0xCE:
        case 0xCF:

        case 0xD0:
        case 0xD1:
        case 0xD2:
        case 0xD3:
        case 0xD4:
        case 0xD5:
        case 0xD6:
        case 0xD7:
        case 0xD8:
        case 0xD9:
        case 0xDA:
        case 0xDB:
        case 0xDC:
        case 0xDD:
        case 0xDE:
        case 0xDF:

        case 0xE0:
        case 0xE1:
        case 0xE2:
        case 0xE3:
        case 0xE4:
        case 0xE5:
        case 0xE6:
        case 0xE7:
        case 0xE8:
        case 0xE9:
        case 0xEA:
        case 0xEB:
        case 0xEC:
        case 0xED:
        case 0xEE:
        case 0xEF:

        case 0xF0:
        case 0xF1:
        case 0xF2:
        case 0xF3:
        case 0xF4:
        case 0xF5:
        case 0xF6:
        case 0xF7:
        case 0xF8:
        case 0xF9:
        case 0xFA:
        case 0xFB:
        case 0xFC:
        case 0xFD:
        case 0xFE:
        case 0xFF:
    }
}

char *gb_disassemble(u8 *pc, struct gb_data *data) {
    switch(data->PC) {
        case 0x00: return "NOP";
        case 0x01: return "LD BC, nn";
        case 0x02: return "LD (BC), A";
        case 0x03: return "INC BC";
        case 0x04: return "INC B";
        case 0x05: return "DEC B";
        case 0x06: return "LD B, n";
        case 0x07: return "RLCA";
        case 0x08: return "LD (nn), SP";
        case 0x09: return "ADD HL, BC";
        case 0x0A: return "LD A, (BC)";
        case 0x0B: return "DEC BC";
        case 0x0C: return "INC C";
        case 0x0D: return "DEC C";
        case 0x0E: return "LD C, n";
        case 0x0F: return "RRCA";

        case 0x10:
        case 0x11:
        case 0x12:
        case 0x13:
        case 0x14:
        case 0x15:
        case 0x16:
        case 0x17:
        case 0x18:
        case 0x19:
        case 0x1A:
        case 0x1B:
        case 0x1C:
        case 0x1D:
        case 0x1E:
        case 0x1F:

        case 0x20:
        case 0x21:
        case 0x22:
        case 0x23:
        case 0x24:
        case 0x25:
        case 0x26:
        case 0x27:
        case 0x28:
        case 0x29:
        case 0x2A:
        case 0x2B:
        case 0x2C:
        case 0x2D:
        case 0x2E:
        case 0x2F:

        case 0x30:
        case 0x31:
        case 0x32:
        case 0x33:
        case 0x34:
        case 0x35:
        case 0x36:
        case 0x37:
        case 0x38:
        case 0x39:
        case 0x3A:
        case 0x3B:
        case 0x3C:
        case 0x3D:
        case 0x3E:
        case 0x3F:

        case 0x40: return LD_r_r(B, B);
        case 0x41: return LD_r_r(B, C);
        case 0x42: return LD_r_r(B, D);
        case 0x43: return LD_r_r(B, E);
        case 0x44: return LD_r_r(B, H);
        case 0x45: return LD_r_r(B, L);
        case 0x46: return LD_r_irr(B, HL);
        case 0x47: return LD_r_r(B, A);
        case 0x48: return LD_r_r(C, B);
        case 0x49: return LD_r_r(C, C);
        case 0x4A: return LD_r_r(C, D);
        case 0x4B: return LD_r_r(C, E);
        case 0x4C: return LD_r_r(C, H);
        case 0x4D: return LD_r_r(C, L);
        case 0x4E: return LD_r_irr(C, HL);
        case 0x4F: return LD_r_r(C, A);

        case 0x50: return LD_r_r(D, B);
        case 0x51: return LD_r_r(D, C);
        case 0x52: return LD_r_r(D, D);
        case 0x53: return LD_r_r(D, E);
        case 0x54: return LD_r_r(D, H);
        case 0x55: return LD_r_r(D, L);
        case 0x56: return LD_r_irr(D, HL);
        case 0x57: return LD_r_r(D, A);
        case 0x58: return LD_r_r(E, B);
        case 0x59: return LD_r_r(E, C);
        case 0x5A: return LD_r_r(E, D);
        case 0x5B: return LD_r_r(E, E);
        case 0x5C: return LD_r_r(E, H);
        case 0x5D: return LD_r_r(E, L);
        case 0x5E: return LD_r_irr(E, HL);
        case 0x5F: return LD_r_r(E, A);

        case 0x60:
        case 0x61:
        case 0x62:
        case 0x63:
        case 0x64:
        case 0x65:
        case 0x66:
        case 0x67:
        case 0x68:
        case 0x69:
        case 0x6A:
        case 0x6B:
        case 0x6C:
        case 0x6D:
        case 0x6E:
        case 0x6F:

        case 0x70:
        case 0x71:
        case 0x72:
        case 0x73:
        case 0x74:
        case 0x75:
        case 0x76:
        case 0x77:
        case 0x78:
        case 0x79:
        case 0x7A:
        case 0x7B:
        case 0x7C:
        case 0x7D:
        case 0x7E:
        case 0x7F:

        case 0x80:
        case 0x81:
        case 0x82:
        case 0x83:
        case 0x84:
        case 0x85:
        case 0x86:
        case 0x87:
        case 0x88:
        case 0x89:
        case 0x8A:
        case 0x8B:
        case 0x8C:
        case 0x8D:
        case 0x8E:
        case 0x8F:

        case 0x90:
        case 0x91:
        case 0x92:
        case 0x93:
        case 0x94:
        case 0x95:
        case 0x96:
        case 0x97:
        case 0x98:
        case 0x99:
        case 0x9A:
        case 0x9B:
        case 0x9C:
        case 0x9D:
        case 0x9E:
        case 0x9F:

        case 0xA0:
        case 0xA1:
        case 0xA2:
        case 0xA3:
        case 0xA4:
        case 0xA5:
        case 0xA6:
        case 0xA7:
        case 0xA8:
        case 0xA9:
        case 0xAA:
        case 0xAB:
        case 0xAC:
        case 0xAD:
        case 0xAE:
        case 0xAF:

        case 0xB0:
        case 0xB1:
        case 0xB2:
        case 0xB3:
        case 0xB4:
        case 0xB5:
        case 0xB6:
        case 0xB7:
        case 0xB8:
        case 0xB9:
        case 0xBA:
        case 0xBB:
        case 0xBC:
        case 0xBD:
        case 0xBE:
        case 0xBF:

        case 0xC0:
        case 0xC1:
        case 0xC2:
        case 0xC3:
        case 0xC4:
        case 0xC5:
        case 0xC6:
        case 0xC7:
        case 0xC8:
        case 0xC9:
        case 0xCA:
        case 0xCB:
        case 0xCC:
        case 0xCD:
        case 0xCE:
        case 0xCF:

        case 0xD0:
        case 0xD1:
        case 0xD2:
        case 0xD3:
        case 0xD4:
        case 0xD5:
        case 0xD6:
        case 0xD7:
        case 0xD8:
        case 0xD9:
        case 0xDA:
        case 0xDB:
        case 0xDC:
        case 0xDD:
        case 0xDE:
        case 0xDF:

        case 0xE0:
        case 0xE1:
        case 0xE2:
        case 0xE3:
        case 0xE4:
        case 0xE5:
        case 0xE6:
        case 0xE7:
        case 0xE8:
        case 0xE9:
        case 0xEA:
        case 0xEB:
        case 0xEC:
        case 0xED:
        case 0xEE:
        case 0xEF:

        case 0xF0:
        case 0xF1:
        case 0xF2:
        case 0xF3:
        case 0xF4:
        case 0xF5:
        case 0xF6:
        case 0xF7:
        case 0xF8:
        case 0xF9:
        case 0xFA:
        case 0xFB:
        case 0xFC:
        case 0xFD:
        case 0xFE:
        case 0xFF:
    }
}