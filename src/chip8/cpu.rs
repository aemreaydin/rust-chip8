use std::time::Duration;

use super::display;
use rand::Rng;

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

trait FontMemStart {
    const FONT_MEM_START: usize = 0x050;
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
    pub display: display::Display,
}
impl FontMemStart for CPU {}

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
            display: display::Display::new(640, 320),
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
        for (ind, font) in FONTS.iter().enumerate() {
            self.memory[CPU::FONT_MEM_START + ind] = *font;
        }
    }

    fn load_rom_into_memory(&mut self, rom_buf: &[u8]) {
        for (ind, byte) in rom_buf.iter().enumerate() {
            self.memory[(self.prog_counter as usize) + ind] = *byte;
        }
    }

    fn convert_rom_to_opcodes(rom_buf: &[u8]) -> Vec<u16> {
        let mut opcodes: Vec<u16> = Vec::new();
        for index in 0..(rom_buf.len() / 2) {
            let val0 = rom_buf[2 * index];
            let val1 = rom_buf[2 * index + 1];
            let opcode = ((val0 as u16) << 8) | val1 as u16;
            opcodes.push(opcode);
        }
        opcodes
    }

    fn fetch_current_instruction(&mut self) -> u16 {
        ((self.memory[self.prog_counter as usize] as u16) << 8)
            | (self.memory[(self.prog_counter + 1) as usize] as u16)
    }

    pub fn run(&mut self) {
        'running: loop {
            let opcode = self.fetch_current_instruction();
            println!("{:04X}", opcode);
            self.run_instruction(opcode);

            let should_break = self.display.update();
            if should_break {
                break 'running;
            }

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }

    pub fn run_instruction(&mut self, opcode: u16) {
        let (op0, op1, op2, op3): (u8, u8, u8, u8) = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = ((op1 as u16) << 8) | ((op2 as u16) << 4) | (op3 as u16);
        let nn: u8 = ((op2 as u8) << 4) | (op3 as u8);
        let n = op3;
        let vx = op1;
        let vy = op2;

        match (op0, op1, op2, op3) {
            (0x0, 0x0, 0xE, 0x0) => self.clear_display(),
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump_to_address(nnn),
            (0x2, _, _, _) => self.call_subroutine_at_address(nnn),
            (0x3, _, _, _) => self.skip_if_vx_eq_nn(vx, nn),
            (0x4, _, _, _) => self.skip_if_vx_neq_nn(vx, nn),
            (0x5, _, _, _) => self.skip_if_vx_eq_vy(vx, vy),
            (0x6, _, _, _) => self.set_vx_to_nn(vx, nn),
            (0x7, _, _, _) => self.add_vx_nn(vx, nn),
            (0x8, _, _, 0x0) => self.set_vx_to_vy(vx, vy),
            (0x8, _, _, 0x1) => self.set_vx_to_vx_or_vy(vx, vy),
            (0x8, _, _, 0x2) => self.set_vx_to_vx_and_vy(vx, vy),
            (0x8, _, _, 0x3) => self.set_vx_to_vx_xor_vy(vx, vy),
            (0x8, _, _, 0x4) => self.add_vx_vy(vx, vy),
            (0x8, _, _, 0x5) => self.sub_vx_vy(vx, vy),
            (0x8, _, _, 0x6) => self.shift_vx_right(vx),
            (0x8, _, _, 0x7) => self.sub_vy_vx(vx, vy),
            (0x8, _, _, 0xE) => self.shift_vx_left(vx),
            (0x9, _, _, _) => self.skip_if_vx_neq_vy(vx, vy),
            (0xA, _, _, _) => self.set_ind_reg_to_address(nnn),
            (0xB, _, _, _) => self.jump_to_v0_plus_address(nnn),
            (0xC, _, _, _) => self.set_vx_to_rnd_and_nn(vx, nn),
            (0xD, _, _, _) => self.display_sprite(vx, vy, n),
            (0xE, _, 0x9, 0xE) => self.skip_if_key_eq_vx_pressed(vx),
            (0xE, _, 0xA, 0x1) => self.skip_if_key_eq_vx_not_pressed(vx),
            (0xF, _, 0x0, 0x7) => self.set_vx_to_delay_timer(vx),
            (0xF, _, 0x0, 0xA) => self.set_vx_to_key_press(vx),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer_to_vx(vx),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer_to_vx(vx),
            (0xF, _, 0x1, 0xE) => self.add_ind_reg_vx(vx),
            (0xF, _, 0x2, 0x9) => self.set_ind_reg_to_loc_of_sprite_for_digit_vx(vx),
            (0xF, _, 0x3, 0x3) => self.store_bcd_vx_in_ind_reg(vx),
            (0xF, _, 0x5, 0x5) => self.store_v_reg_in_memory_from_ind_reg(vx),
            (0xF, _, 0x6, 0x5) => self.read_v_reg_from_ind_reg(vx),
            _ => println!("NEXT_INST"),
        }
    }
    // 00E0
    fn clear_display(&mut self) {
        self.display.clear();
        self.prog_counter += 2;
    }
    // 00EE
    fn return_from_subroutine(&mut self) {
        self.stack_ptr -= 1;
        self.prog_counter = self.stack[(self.stack_ptr as usize)];
    }
    // 1NNN
    fn jump_to_address(&mut self, address: u16) {
        self.prog_counter = address;
    }
    // 2NNN
    fn call_subroutine_at_address(&mut self, address: u16) {
        // Store the program counter in the stack
        self.stack[(self.stack_ptr as usize)] = self.prog_counter;
        self.stack_ptr += 1;
        self.prog_counter = address;
    }
    // 3XNN
    fn skip_if_vx_eq_nn(&mut self, vx: u8, nn: u8) {
        if self.v_reg[(vx as usize)] == nn {
            self.prog_counter += 2;
        }
        self.prog_counter += 2;
    }
    // 4XNN
    fn skip_if_vx_neq_nn(&mut self, vx: u8, nn: u8) {
        if self.v_reg[vx as usize] != nn {
            self.prog_counter += 2;
        }
        self.prog_counter += 2;
    }
    // 5XY0
    fn skip_if_vx_eq_vy(&mut self, vx: u8, vy: u8) {
        if self.v_reg[vx as usize] == self.v_reg[vy as usize] {
            self.prog_counter += 2;
        }
        self.prog_counter += 2;
    }
    // 6XNN
    fn set_vx_to_nn(&mut self, vx: u8, nn: u8) {
        self.v_reg[vx as usize] = nn;
        self.prog_counter += 2;
    }
    // 7XNN
    fn add_vx_nn(&mut self, vx: u8, nn: u8) {
        self.v_reg[vx as usize] = self.v_reg[vx as usize].wrapping_add(nn);
        self.prog_counter += 2;
    }
    // 8XY0
    fn set_vx_to_vy(&mut self, vx: u8, vy: u8) {
        self.v_reg[vx as usize] = self.v_reg[vy as usize];
        self.prog_counter += 2;
    }
    // 8XY1
    fn set_vx_to_vx_or_vy(&mut self, vx: u8, vy: u8) {
        self.v_reg[vx as usize] |= self.v_reg[vy as usize];
        self.prog_counter += 2;
    }
    // 8XY2
    fn set_vx_to_vx_and_vy(&mut self, vx: u8, vy: u8) {
        self.v_reg[vx as usize] &= self.v_reg[vy as usize];
        self.prog_counter += 2;
    }
    // 8XY3
    fn set_vx_to_vx_xor_vy(&mut self, vx: u8, vy: u8) {
        self.v_reg[vx as usize] ^= self.v_reg[vy as usize];
        self.prog_counter += 2;
    }
    // 8XY4
    fn add_vx_vy(&mut self, vx: u8, vy: u8) {
        let val_x = self.v_reg[vx as usize];
        let val_y = self.v_reg[vy as usize];

        let (sum, did_overflow) = val_x.overflowing_add(val_y);
        self.v_reg[vx as usize] = sum;
        self.v_reg[0xF] = if did_overflow { 1 } else { 0 };
        self.prog_counter += 2;
    }
    // 8XY5
    fn sub_vx_vy(&mut self, vx: u8, vy: u8) {
        let val_x = self.v_reg[vx as usize];
        let val_y = self.v_reg[vy as usize];

        let (sub, did_overflow) = val_x.overflowing_sub(val_y);
        self.v_reg[vx as usize] = sub;
        self.v_reg[0xF] = if did_overflow { 1 } else { 0 };
        self.prog_counter += 2;
    }
    // 8XY6
    fn shift_vx_right(&mut self, vx: u8) {
        let val_x = self.v_reg[vx as usize];
        self.v_reg[0xF] = val_x & 1;
        self.v_reg[vx as usize] = val_x >> 1;
        self.prog_counter += 2;
    }
    // 8XY7
    fn sub_vy_vx(&mut self, vx: u8, vy: u8) {
        let val_x = self.v_reg[vx as usize];
        let val_y = self.v_reg[vy as usize];

        let (sub, did_overflow) = val_y.overflowing_sub(val_x);
        self.v_reg[vx as usize] = sub;
        self.v_reg[0xF] = if did_overflow { 1 } else { 0 };
        self.prog_counter += 2;
    }
    // 8XYE
    fn shift_vx_left(&mut self, vx: u8) {
        let val_x = self.v_reg[vx as usize];
        self.v_reg[0xF] = (val_x >> 7) & 1;
        self.v_reg[vx as usize] = val_x << 1;
        self.prog_counter += 2;
    }
    // 9XY0
    fn skip_if_vx_neq_vy(&mut self, vx: u8, vy: u8) {
        if self.v_reg[vx as usize] != self.v_reg[vy as usize] {
            self.prog_counter += 2;
        }
        self.prog_counter += 2;
    }
    // ANNN
    fn set_ind_reg_to_address(&mut self, address: u16) {
        self.i_reg = address;
        self.prog_counter += 2;
    }
    // BNNN
    fn jump_to_v0_plus_address(&mut self, address: u16) {
        self.prog_counter = (self.v_reg[0] as u16) + address;
    }
    // CXNN
    fn set_vx_to_rnd_and_nn(&mut self, vx: u8, nn: u8) {
        let mut rng = rand::thread_rng();
        let rnd: u8 = rng.gen();
        self.v_reg[vx as usize] = rnd & nn;
        self.prog_counter += 2;
    }
    // DXYN
    fn display_sprite(&mut self, vx: u8, vy: u8, n: u8) {
        let x_coords = self.v_reg[vx as usize];
        let y_coords = self.v_reg[vy as usize];
        self.v_reg[0xF] = 0;

        for row in 0..n {
            let sprite = self.memory[(self.i_reg as usize) + row as usize];
            let y_coord = (y_coords + row) as u32 % display::BASE_HEIGHT;
            for bit in 0..8u8 {
                let x_coord = (x_coords + bit) as u32 % display::BASE_WIDTH;
                let pixel = self.display.get_pixel(x_coord, y_coord);
                let sprite_bit = sprite >> (7 - bit) & 1;
                self.v_reg[0x0F] = pixel & sprite_bit;
                self.display.set_pixel(x_coord, y_coord, pixel ^ sprite_bit);
            }
        }

        self.display.draw();
        self.prog_counter += 2;
    }
    // EX9E
    fn skip_if_key_eq_vx_pressed(&mut self, _vx: u8) {
        self.prog_counter += 2;
    }
    // EXA1
    fn skip_if_key_eq_vx_not_pressed(&mut self, _vx: u8) {
        self.prog_counter += 2;
    }
    // FX07
    fn set_vx_to_delay_timer(&mut self, vx: u8) {
        self.v_reg[vx as usize] = self.delay_reg;
        self.prog_counter += 2;
    }
    // FX0A
    fn set_vx_to_key_press(&mut self, _vx: u8) {
        self.prog_counter += 2;
    }
    // FX15
    fn set_delay_timer_to_vx(&mut self, vx: u8) {
        self.delay_reg = self.v_reg[vx as usize];
        self.prog_counter += 2;
    }
    // FX18
    fn set_sound_timer_to_vx(&mut self, vx: u8) {
        self.sound_reg = self.v_reg[vx as usize];
        self.prog_counter += 2;
    }
    // FX1E
    fn add_ind_reg_vx(&mut self, vx: u8) {
        self.i_reg += self.v_reg[vx as usize] as u16;
        self.prog_counter += 2;
    }
    // FX29
    fn set_ind_reg_to_loc_of_sprite_for_digit_vx(&mut self, vx: u8) {
        let x = self.v_reg[vx as usize];
        self.i_reg = (x as u16) * 5;
        self.prog_counter += 2;
    }
    // FX33
    fn store_bcd_vx_in_ind_reg(&mut self, vx: u8) {
        let ones = self.v_reg[vx as usize] % 10;
        let tens = (self.v_reg[vx as usize] / 10) % 10;
        let hundreds = (self.v_reg[vx as usize] / 100) % 10;
        self.memory[(self.i_reg) as usize] = hundreds;
        self.memory[(self.i_reg + 1) as usize] = tens;
        self.memory[(self.i_reg + 2) as usize] = ones;
        self.prog_counter += 2;
    }
    // FX55
    fn store_v_reg_in_memory_from_ind_reg(&mut self, vx: u8) {
        for ind in 0..=(vx as usize) {
            self.memory[(self.i_reg as usize) + ind] = self.v_reg[ind];
        }
        self.prog_counter += 2;
    }
    // FX65
    fn read_v_reg_from_ind_reg(&mut self, vx: u8) {
        for ind in 0..=(vx as usize) {
            self.v_reg[ind] = self.memory[(self.i_reg as usize) + ind];
        }
        self.prog_counter += 2;
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
    fn converts_opcode() {
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
    #[test]
    fn calls_and_returns_from_subroutine() {
        let mut cpu = CPU::new(&[]);
        let addr = 0x300;
        cpu.call_subroutine_at_address(addr);
        assert_eq!(cpu.prog_counter, addr);
        cpu.return_from_subroutine();
        assert_eq!(cpu.prog_counter, 0x200);
    }
    #[test]
    fn skips_if_vx_eq_nn() {
        let mut cpu = CPU::new(&[]);
        let val = 0xCC;
        cpu.v_reg[0] = val;
        // Doesn't skip
        cpu.skip_if_vx_eq_nn(0, 0xCD);
        assert_eq!(cpu.prog_counter, 0x200);
        // Skip
        cpu.skip_if_vx_eq_nn(0, val);
        assert_eq!(cpu.prog_counter, 0x202);
    }
    #[test]
    fn skips_if_vx_neq_nn() {
        let mut cpu = CPU::new(&[]);
        let val = 0xCC;
        cpu.v_reg[0] = val;
        // Doesn't skip
        cpu.skip_if_vx_neq_nn(0, val);
        assert_eq!(cpu.prog_counter, 0x200);
        // Skip
        cpu.skip_if_vx_neq_nn(0, 0xCD);
        assert_eq!(cpu.prog_counter, 0x202);
    }
    #[test]
    fn skips_if_vx_eq_vy() {
        let mut cpu = CPU::new(&[]);
        let val = 0xCC;
        let vx: u8 = 0;
        let vy: u8 = 1;
        // Doesn't skip
        cpu.v_reg[(vx as usize)] = val;
        cpu.skip_if_vx_eq_vy(vx, vy);
        assert_eq!(cpu.prog_counter, 0x200);
        // Skip
        cpu.v_reg[(vy as usize)] = val;
        cpu.skip_if_vx_eq_vy(vx, vy);
        assert_eq!(cpu.prog_counter, 0x202);
    }
    #[test]
    fn sets_vx_to_nn() {
        let mut cpu = CPU::new(&[]);
        let val = 0xFF;
        cpu.set_vx_to_nn(0, val);
        assert_eq!(cpu.v_reg[0], val);
    }
    #[test]
    fn adds_vx_nn() {
        let mut cpu = CPU::new(&[]);
        let val = 0x01;
        cpu.v_reg[0] = 0x02;
        cpu.add_vx_nn(0, val);
        assert_eq!(cpu.v_reg[0], 0x03);
    }
    #[test]
    fn sets_vx_to_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x03;
        cpu.set_vx_to_vy(0, 1);
        assert_eq!(cpu.v_reg[0], cpu.v_reg[1]);
    }
    #[test]
    fn sets_vx_to_vx_or_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x03;
        cpu.set_vx_to_vx_or_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0x02 | 0x03);
    }
    #[test]
    fn sets_vx_to_vx_and_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x03;
        cpu.set_vx_to_vx_and_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0x02 & 0x03);
    }
    #[test]
    fn sets_vx_to_vx_xor_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x03;
        cpu.set_vx_to_vx_xor_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0x02 ^ 0x03);
    }
    #[test]
    fn adds_vx_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x03;
        cpu.add_vx_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0x05);
        assert_eq!(cpu.v_reg[0xF], 0);
        cpu.v_reg[0] = 0xFF;
        cpu.v_reg[1] = 0x01;
        cpu.add_vx_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0);
        assert_eq!(cpu.v_reg[0xF], 1);
    }
    #[test]
    fn subs_vx_vy() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x03;
        cpu.v_reg[1] = 0x02;
        cpu.sub_vx_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0x01);
        assert_eq!(cpu.v_reg[0xF], 0);
        cpu.v_reg[0] = 0x00;
        cpu.v_reg[1] = 0x01;
        cpu.sub_vx_vy(0, 1);
        assert_eq!(cpu.v_reg[0], 0xFF);
        assert_eq!(cpu.v_reg[0xF], 1);
    }
    #[test]
    fn shifts_vx_right() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x03;
        cpu.shift_vx_right(0);
        assert_eq!(cpu.v_reg[0], 1);
        assert_eq!(cpu.v_reg[0xF], 1);
    }
    #[test]
    fn subs_vy_vx() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x02;
        cpu.v_reg[1] = 0x04;
        cpu.sub_vy_vx(0, 1);
        assert_eq!(cpu.v_reg[0], 0x02);
        assert_eq!(cpu.v_reg[0xF], 0);
        cpu.v_reg[0] = 0x01;
        cpu.v_reg[1] = 0x00;
        cpu.sub_vy_vx(0, 1);
        assert_eq!(cpu.v_reg[0], 0xFF);
        assert_eq!(cpu.v_reg[0xF], 1);
    }
    #[test]
    fn shifts_vx_left() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x0F;
        cpu.shift_vx_left(0);
        assert_eq!(cpu.v_reg[0], 0x1E);
        assert_eq!(cpu.v_reg[0xF], 0);
        cpu.v_reg[0] = 0xFF;
        cpu.shift_vx_left(0);
        assert_eq!(cpu.v_reg[0xF], 1);
    }
    #[test]
    fn skips_if_vx_neq_vy() {
        let mut cpu = CPU::new(&[]);
        let val = 0xCC;
        cpu.v_reg[0] = val;
        cpu.v_reg[1] = val;
        // Doesn't skip
        cpu.skip_if_vx_neq_vy(0, 1);
        assert_eq!(cpu.prog_counter, 0x200);
        // Skip
        cpu.v_reg[1] = val + 1;
        cpu.skip_if_vx_neq_vy(0, 1);
        assert_eq!(cpu.prog_counter, 0x202);
    }
    #[test]
    fn sets_ind_reg_to_address() {
        let mut cpu = CPU::new(&[]);
        let address = 0x0ABC;
        cpu.set_ind_reg_to_address(address);
        assert_eq!(cpu.i_reg, address);
    }
    #[test]
    fn jumps_to_v0_plus_address() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0xFF;
        let address = 0xABC;
        cpu.jump_to_v0_plus_address(address);
        assert_eq!(cpu.prog_counter, 0xFF + address);
    }
    #[test]
    fn sets_vx_to_rnd_and_nn() {
        let mut cpu = CPU::new(&[]);
        cpu.set_vx_to_rnd_and_nn(0, 0x0F);
        assert_eq!(cpu.v_reg[0] & 0xF0, 0);
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn displays_sprite() {
        todo!();
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn skips_if_key_eq_vx_pressed() {
        todo!();
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn skips_if_key_eq_vx_not_pressed() {
        todo!();
    }
    #[test]
    fn sets_vx_to_delay_timer() {
        let mut cpu = CPU::new(&[]);
        cpu.delay_reg = 0x03;
        cpu.set_vx_to_delay_timer(0);
        assert_eq!(cpu.v_reg[0], cpu.delay_reg);
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn sets_vx_to_key_press() {
        todo!();
    }
    #[test]
    fn sets_delay_timer_to_vx() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x03;
        cpu.set_delay_timer_to_vx(0);
        assert_eq!(cpu.v_reg[0], cpu.delay_reg);
    }
    #[test]
    fn sets_sound_timer_to_vx() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 0x03;
        cpu.set_sound_timer_to_vx(0);
        assert_eq!(cpu.v_reg[0], cpu.sound_reg);
    }
    #[test]
    fn adds_ind_reg_vx() {
        let mut cpu = CPU::new(&[]);
        cpu.i_reg = 0x02;
        cpu.v_reg[0] = 0x03;
        cpu.add_ind_reg_vx(0);
        assert_eq!(cpu.i_reg, 0x05);
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn sets_ind_reg_to_loc_of_sprite_for_digit_vx() {
        todo!();
    }
    #[test]
    fn stores_bcd_vx_in_ind_reg() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 143;
        cpu.store_bcd_vx_in_ind_reg(0);
        assert_eq!(cpu.memory[cpu.i_reg as usize], 1);
        assert_eq!(cpu.memory[(cpu.i_reg + 1) as usize], 4);
        assert_eq!(cpu.memory[(cpu.i_reg + 2) as usize], 3);
    }
    #[test]
    fn stores_v_reg_in_memory_from_ind_reg() {
        let mut cpu = CPU::new(&[]);
        cpu.v_reg[0] = 143;
        cpu.v_reg[1] = 255;
        cpu.v_reg[2] = 12;
        cpu.store_v_reg_in_memory_from_ind_reg(2);
        assert_eq!(cpu.memory[cpu.i_reg as usize], 143);
        assert_eq!(cpu.memory[(cpu.i_reg + 1) as usize], 255);
        assert_eq!(cpu.memory[(cpu.i_reg + 2) as usize], 12);
    }
    #[test]
    fn reads_v_reg_from_ind_reg() {
        let mut cpu = CPU::new(&[]);
        cpu.memory[cpu.i_reg as usize] = 143;
        cpu.memory[(cpu.i_reg + 1) as usize] = 255;
        cpu.memory[(cpu.i_reg + 2) as usize] = 3;
        cpu.read_v_reg_from_ind_reg(2);
        assert_eq!(cpu.v_reg[0], 143);
        assert_eq!(cpu.v_reg[1], 255);
        assert_eq!(cpu.v_reg[2], 3);
    }
}
