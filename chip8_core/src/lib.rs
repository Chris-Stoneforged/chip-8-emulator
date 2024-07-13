use rand::random;
use std::fmt::Display;

pub const SCREEN_WIDTH: usize = 64; 
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [ 
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Op {
    op: u16,
    d1: u8,
    d2: u8,
    d3: u8,
    d4: u8, 
}

impl Op {
    fn from_opcode(opcode: u16) -> Self{
        Self {
            op: opcode,
            d1: ((opcode & 0xF000) >> 12) as u8,
            d2: ((opcode & 0x0F00) >> 8) as u8, 
            d3: ((opcode & 0x00F0) >> 4) as u8, 
            d4: (opcode & 0x000F) as u8,
        }
    }

    fn get_nn(&self) -> u8 {
        (self.op & 0xFF) as u8
    }

    fn get_nnn(&self) -> u16 {
        self.op & 0xFFF
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.op)
    }
}

struct Stack {
    sp: u16,
    stack: [u16; STACK_SIZE],
}

impl Stack {
    pub fn new() -> Self {
        Self {
            sp: 0,
            stack: [0; STACK_SIZE],
        }
    }

    fn clear(&mut self) {
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
    }

    fn push(&mut self, val: u16) { 
        self.stack[self.sp as usize] = val; 
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 { 
        self.sp -= 1;
        self.stack[self.sp as usize]
    }       
}

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
    stack: Stack,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            stack: Stack::new(),
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]; self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0; self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.stack.clear();
    }

    pub fn tick_timers(&mut self) { 
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
            if self.st == 0 {
                // TODO: Sound
            }  
        }    
    }

    pub fn tick(&mut self) { 
        // Fetch
        let op = self.fetch();
        self.execute(op)
    }

    fn fetch(&mut self) -> Op {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16; 
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        Op::from_opcode(op)
    }

    fn execute(&mut self, op: Op) { 
        let Op{ d1, d2, d3, d4, ..} = op;

        match (d1, d2, d3, d4) {
            // NOP
            (0, 0, 0, 0) => return,
            // CLEAR SCREEN
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RETURN FROM SUBROUTINE
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.stack.pop(); 
                self.pc = ret_addr;
            },
            // JUMP TO NNN
            (1, _, _, _) => {
                self.pc = op.get_nnn();
            },
            // CALL SUBROUTINE NNN
            (2, _, _, _) => {
                let nnn = op.get_nnn(); 
                self.stack.push(self.pc); 
                self.pc = nnn;
            },
            // SKIP IF VX == NN
            (3, _, _, _) => {
                if self.v_reg[d2 as usize] == op.get_nn() {
                    self.pc += 2;
                }
            },    
            // SKIP IF VX != NN
            (4, _, _, _) => {
                if self.v_reg[d2 as usize] != op.get_nn() {
                self.pc += 2;
                } 
            },
            // SKIP IF VX == VY
            (5, _, _, 0) => {
                if self.v_reg[d2 as usize] == self.v_reg[d3 as usize] {
                    self.pc += 2;
                } 
            },
            // SET VX = NN
            (6, _, _, _) => {
                self.v_reg[d2 as usize] = op.get_nn();
            },
            // SET VX += NN
            (7, _, _, _) => {
                let x = d2 as usize;
                self.v_reg[x] = self.v_reg[x].wrapping_add(op.get_nn());
            },
            // SET VX = VY
            (8, _, _, 0) => {
                self.v_reg[d2 as usize] = self.v_reg[d3 as usize];
            },
            // SET VX |= VY
            (8, _, _, 1) => {
                self.v_reg[d2 as usize] |= self.v_reg[d3 as usize];
            },
            // SET VX &= VY
            (8, _, _, 2) => {
                self.v_reg[d2 as usize] &= self.v_reg[d3 as usize];
            },
            // SET VX ^= VY
            (8, _, _, 3) => {
                self.v_reg[d2 as usize] ^= self.v_reg[d3 as usize];
            },
            // SET VX += VY
            (8, _, _, 4) => {
                let x = d2 as usize;
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[d3 as usize]); 
                let new_vf = if carry { 1 } else { 0 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // SET VX -= VY
            (8, _, _, 5) => {
                let x = d2 as usize;
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[d3 as usize]); 
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // SET VX >>= 1
            (8, _, _, 6) => {
                let x = d2 as usize;
                let lsb = self.v_reg[x] & 1; 
                self.v_reg[x] >>= 1; 
                self.v_reg[0xF] = lsb;
            },
            // SET VX = VY - VX
            (8, _, _, 7) => {
                let x = d2 as usize;
                let (new_vx, borrow) = self.v_reg[d3 as usize].overflowing_sub(self.v_reg[x]); 
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // SET VX <<= 1
            (8, _, _, 0xE) => {
                let x = d2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1; 
                self.v_reg[x] <<= 1; self.v_reg[0xF] = msb;
            },
            // SKIP IF VX == VY
            (9, _, _, 0) => {
                if self.v_reg[d2 as usize] != self.v_reg[d3 as usize] {
                    self.pc += 2;
                }
            },
            // SET I = NNN
            (0xA, _, _, _) => {
                self.i_reg = op.get_nnn();
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                self.pc = (self.v_reg[0] as u16) + op.get_nnn();
            },
            // SET VX = rand() & NN
            (0xC, _, _, _) => {
                let rng: u8 = random(); 
                self.v_reg[d2 as usize] = rng & op.get_nn();
            },
            // DRAW
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[d2 as usize] as u16;
                let y_coord = self.v_reg[d3 as usize] as u16;
                let num_rows = d4 as u16;
                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                self.v_reg[0xF] = if flipped { 1 } else { 0 }
            },
            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
                let vx = self.v_reg[d2 as usize];
                let key = self.keys[vx as usize]; 
                if key {
                    self.pc += 2;
                } 
            },
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => {
                let vx = self.v_reg[d2 as usize];
                let key = self.keys[vx as usize]; 
                if !key {
                    self.pc += 2;
                } 
            },
            // SET VX = DT
            (0xF, _, 0, 7) => {
                self.v_reg[d2 as usize] = self.dt;
            },
            // WAIT FOR KEY PRESS
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] { 
                        self.v_reg[d2 as usize] = i as u8; 
                        pressed = true;
                        break;
                    }
                }
                
                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            },
            // SET DT = VX
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[d2 as usize];
            },
            // SET ST = VX
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[d2 as usize];
            },
            // SET I += VX
            (0xF, _, 1, 0xE) => {
                let vx = self.v_reg[d2 as usize] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // I = FONT
            (0xF, _, 2, 9) => {
                let c = self.v_reg[d2 as usize] as u16; 
                self.i_reg = c * 5;
            },
            // BINARY CODED DECIMAL (CONVERT BINARY TO BASE10 DIGITS)
            (0xF, _, 3, 3) => {
                let vx = self.v_reg[d2 as usize] as f32;
                
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds; 
                self.ram[(self.i_reg + 1) as usize] = tens; 
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize; 
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize; 
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                } 
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op), 
        }
    }
}