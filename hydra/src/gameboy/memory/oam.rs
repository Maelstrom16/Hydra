use crate::common::errors::HydraIOError;

pub struct OAM {
    inner: [u8; 0x100],
}

pub const ADDRESS_OFFSET: usize = 0xFE00;

impl OAM {
    pub fn new() -> Self {
        OAM { inner: [0; 0x100] }
    }

    pub fn read(&self, address: usize) -> Result<u8, HydraIOError> {
        Ok(self.inner[address])
    }

    pub fn write(&mut self, address: usize, value: u8) -> Result<(), HydraIOError> {
        Ok(self.inner[address] = value)
    }

    fn corruption(&mut self, line: usize) {
        // TODO: implement
        // Corrupt first value in line
        let index = line - ADDRESS_OFFSET;
        let a = self.inner[index]; // Current first value
        let b = self.inner[index - 0x8]; // First value from preceding line
        let c = self.inner[index - 0x6]; // Third value from preceding line

        let read_corruption = true; // TODO: replace with actual logic
        if read_corruption {
            // Apply extra corruption if incrementing or decrementing
            let idu_active = true; // TODO: replace with actual logic
            // Only apply corruption if
            if idu_active && (line >= 0xFE20 && line < 0xFE98) {
                let d = self.inner[index - 0x10]; // First value from preceding preceding line
                self.inner[index - 0x8] = (b & (a | c | d)) | (a & c & d);

                // Copy all values from preceding line to its surrounding lines
                for i in (index)..=(index + 0x7) {
                    self.inner[i] = self.inner[i - 0x8];
                    self.inner[i - 0x10] = self.inner[i - 0x8];
                }
            }
            // Standard read corruption
            self.inner[index] = b | (a & c);
        } else {
            self.inner[index] = ((a ^ c) & (b ^ c)) ^ c;
        }

        // Copy last three values from preceding line
        for i in (index + 0x5)..=(index + 0x7) {
            self.inner[i] = self.inner[i - 0x8];
        }
    }
}
