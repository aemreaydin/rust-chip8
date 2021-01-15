#[derive(Debug)]
pub enum OpCodes {
    _0NNN(u16),
    _00E0(u16),
    _00EE(u16),
    _1NNN(u16),
    _2NNN(u16),
    _3XNN(u16),
}

#[derive(Debug)]
pub struct CPU {
    pub memory: [u8; 4096],
    pub v_reg: [u8; 16],
    pub i_reg: u16,
    pub delay_reg: u8,
    pub sound_reg: u8,
    pub prog_counter: u16,
    pub stack_ptr: u8,
    pub stack: [u16; 16],
    pub opcodes: Vec<u16>,
}

const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl CPU {
    pub fn new(rom_buf: &[u8]) -> Self {
        let opcodes = CPU::convert_rom_to_opcodes(rom_buf);
        let mut cpu = CPU {
            memory: [0; 4096],
            v_reg: [0; 16],
            i_reg: 0,
            delay_reg: 0,
            sound_reg: 0,
            prog_counter: 0x200,
            stack_ptr: 0,
            stack: [0; 16],
            opcodes, // Is used for debugging purposes
        };
        // Initialize fonts in the interpreter btw. 0x000-0x1FF
        // Fonts will be stored between 0x050-0x09F
        cpu.init_fonts();
        // Load the rom into memory starting from 0x200
        // PC will point to 0x200 initially
        cpu.load_rom_into_memory(rom_buf);
        cpu
    }

    fn init_fonts(&mut self) {
        let mem_start = 0x050;
        for (ind, font) in FONTS.iter().enumerate() {
            self.memory[mem_start + ind] = *font;
        }
    }

    fn load_rom_into_memory(&mut self, rom_buf: &[u8]) {
        for (ind, byte) in rom_buf.iter().enumerate() {
            self.memory[(self.prog_counter as usize) + ind] = *byte;
        }
    }

    pub fn convert_rom_to_opcodes(rom_buf: &[u8]) -> Vec<u16> {
        let mut opcodes: Vec<u16> = Vec::new();
        for index in 0..(rom_buf.len() / 2) {
            let val0 = rom_buf[2 * index];
            let val1 = rom_buf[2 * index + 1];
            let opcode = ((val0 as u16) << 8) | val1 as u16;
            opcodes.push(opcode);
        }
        opcodes
    }

    pub fn run_instruction(&mut self, opcode: u16) {
        let (op0, op1, op2, op3): (u8, u8, u8, u8) = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = ((op1 as u16) << 8) | ((op2 as u16) << 4) | (op3 as u16);
        let nn = ((op2 as u16) << 4) | (op3 as u16);
        let n = op3;
        let vx = op1;
        let vy = op2;

        match (op0, op1, op2, op3) {
            (0x0, 0x0, 0xE, 0x0) => println!("CLR"),
            (0x0, 0x0, 0xE, 0xE) => println!("RET"),
            (0x0, _, _, _) => println!("EXEC_ML_NNN"),
            (0x1, _, _, _) => self.jump_to_address(nnn),
            (0x2, _, _, _) => println!("EXEC_NNN"),
            (0x3, _, _, _) => println!("SKIP_VX_EQ_NN"),
            (0x4, _, _, _) => println!("SKIP_VX_NE_NN"),
            (0x5, _, _, _) => println!("SKIP_VX_EQ_VY"),
            (0x6, _, _, _) => println!("STORE_NN_VX"),
            (0x7, _, _, _) => println!("ADD_NN_VX"),
            (0x8, _, _, 0x0) => println!("STORE_VY_VX"),
            (0x8, _, _, 0x1) => println!("SET_VX_OR_VY"),
            (0x8, _, _, 0x2) => println!("SET_VX_AND_VY"),
            (0x8, _, _, 0x3) => println!("SET_VX_XOR_VY"),
            (0x8, _, _, 0x4) => println!("ADD_VY_VX"),
            (0x8, _, _, 0x5) => println!("SUB_VY_VX"),
            (0x8, _, _, 0x6) => println!("STORE_VY_SR_VX"),
            (0x8, _, _, 0x7) => println!("STORE_VY_SUB_VX"),
            (0x8, _, _, 0xE) => println!("STORE_VY_SL_VX"),
            (0x9, _, _, _) => println!("SKIP_VX_NE_VY"),
            (0xA, _, _, _) => println!("SKIP_VX_NE_VY"),
            (0xB, _, _, _) => println!("SKIP_VX_NE_VY"),
            (0xC, _, _, _) => println!("SKIP_VX_NE_VY"),
            (0xD, _, _, _) => println!("SKIP_VX_NE_VY"),
            (0xE, _, 0x9, 0xE) => println!("KEY_PRESSED_EQ_VX"),
            (0xE, _, 0xA, 0x1) => println!("KEY_NOT_PRESSED_EQ_VX"),
            (0xF, _, 0x0, 0x7) => println!("STORE_DELAY_VX"),
            (0xF, _, 0x0, 0xA) => println!("WAIT_KEY_PRESS_STORE_VX"),
            (0xF, _, 0x1, 0x5) => println!("SET_DELAY_VX"),
            (0xF, _, 0x1, 0x8) => println!("SET_SOUND_VX"),
            (0xF, _, 0x1, 0xE) => println!("ADD_VX_VI"),
            (0xF, _, 0x2, 0x9) => println!("SET_I_SPRITE"),
            (0xF, _, 0x3, 0x3) => println!("STORE_BCD_VI"),
            (0xF, _, 0x5, 0x5) => println!("STORE_V0_TO_VX_VI"),
            (0xF, _, 0x6, 0x5) => println!("FILL_VO_TO_VX"),
            _ => println!("NEXT_INST"),
        }
    }

    fn clear_display(&mut self) {
        todo!();
    }

    fn return_from_subroutine(&mut self) {
        todo!();
    }

    fn jump_to_address(&mut self, address: u16) {
        self.prog_counter = address;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    // This is probably not the right way to do this, oh well...
    fn read_test_opcode() -> Vec<u8> {
        let mut file = File::open("roms/ibm_logo.ch8").unwrap();
        let mut rom_buf = Vec::new();
        file.read_to_end(&mut rom_buf).unwrap();
        rom_buf
    }
    #[test]
    fn opcode_conversion_successful() {
        let test_opcode = read_test_opcode();
        let cpu = CPU::new(&test_opcode);
        // Just test the first few to make sure they're correct
        assert_eq!(
            cpu.opcodes[0..=5],
            vec![0x00E0, 0xA22A, 0x600C, 0x6108, 0xD01F, 0x7009,]
        );
        // And test the last two to make sure it works
        assert_eq!(cpu.opcodes[(cpu.opcodes.len() - 2)..], vec![0x00E0, 0x00E0]);
    }
    #[test]
    fn jumps_to_address() {
        let mut cpu = CPU::new(&[]);
        let addr = 0x300;
        cpu.jump_to_address(addr);
        assert_eq!(cpu.prog_counter, addr);
    }
}
