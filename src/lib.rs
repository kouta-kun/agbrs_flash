#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

#[repr(C, align(4))]
pub struct FLASH_ID(pub [u8; 16]);


/// Identifies the ROM as a FLASH1M Cartridge
#[link_section = ".text"]
#[no_mangle]
pub static FLASH_IDENTIFIER: FLASH_ID = FLASH_ID(*b"FLASH1M_Vnnn\0\0\0\0");

pub struct FlashMemory {
    flash_base: *mut u8,
}

// needed bc otherwise it can't be pub static
unsafe impl Send for FlashMemory {}

unsafe impl Sync for FlashMemory {}

pub static MEMORY: FlashMemory = FlashMemory { flash_base: 0xE000000 as *mut u8 };

// store object length at the end of the cartridge to verify whether the cartridge has been written to and as a checksum
const BACKUP_ADDRESS: usize = 0xFFF0;

impl FlashMemory {
    #[doc(hidden)]
    pub fn get_identifier() -> &'static FLASH_ID {
        agb::println!("{:?}", (&FLASH_IDENTIFIER) as *const FLASH_ID);
        return &FLASH_IDENTIFIER;
    }

    /// Whether the cartridge Flash has a structure written to it
    pub fn have_structure(&self) -> bool {
        return self.verify_struct_len();
    }

    /// Erase the cartridge if it didn't have any structure in it
    pub fn init(&self) {
        FlashMemory::get_identifier();
        unsafe {
            self.send_command(0x90);
            let dev = self.read(0x01);
            let man = self.read(0x00);
            self.send_command(0xF0);
            if !self.verify_struct_len() {
                self.clear_memory();
            }
        }
    }

    #[doc(hidden)]
    fn verify_struct_len(&self) -> bool {
        let len_bytes = self.get_structure_len_bytes();
        let len_bytes_backup = self.get_structure_len_bytes_backup();
        let len = u32::from_ne_bytes(len_bytes);
        return (len_bytes == len_bytes_backup) && len != 0xFFFFFFFF;
    }

    #[doc(hidden)]
    unsafe fn clear_memory(&self) {
        self.send_command(0x80);
        self.send_command(0x10);
        while self.read(0x00) != 0xFF {
            // agb::println!("0x00: {}",self.read(0x00));
        }
        // agb::println!("{:#02x} {:#02x}", dev, man);
    }

    #[link_section = ".iwram"]
    #[doc(hidden)]
    fn read(&self, byte: usize) -> u8 {
        unsafe {
            return (self.flash_base.add(byte)).read_volatile();
        }
    }

    #[doc(hidden)]
    unsafe fn send_command(&self, cmd: u8) {
        self.flash_base.add(0x5555).write_volatile(0xAAu8);
        self.flash_base.add(0x2AAA).write_volatile(0x55u8);
        self.flash_base.add(0x5555).write_volatile(cmd);
    }

    #[doc(hidden)]
    fn write(&self, byte: usize, value: u8) -> bool {
        let w = self._write(byte, value);
        return w;
    }

    #[doc(hidden)]
    fn _write(&self, byte: usize, value: u8) -> bool {
        unsafe {
            for _attempt in 0..3 {
                self.send_command(0xA0);
                self.flash_base.add(byte).write_volatile(value);
                for _i in 0..128 {
                    if self.flash_base.add(byte).read_volatile() == value {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    /// Persist a data structure to the cartridge (needs to be serde serializable)
    pub fn write_structure<T: Serialize>(&self, structure: &T) -> bool {
        unsafe { self.clear_memory(); }
        let structure_ser = postcard::to_allocvec(structure).unwrap();
        agb::println!("{}", structure_ser.len());
        let structure_length: [u8; 4] = (structure_ser.len() as u32).to_ne_bytes();
        for i in 0..4 {
            if !self.write(i + BACKUP_ADDRESS, structure_length[i]) {
                return false;
            }
            if !self.write(i, structure_length[i]) {
                return false;
            }
        }
        for i in 0..(structure_ser.len()) {
            if !self.write(i + 4, structure_ser[i]) {
                return false;
            }
        }
        return true;
    }

    /// Read a structure from cartridge flash (needs to be serde deserializable) or return a None if there isn't a structure
    pub fn read_structure<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        let len = self.get_structure_length() as usize;
        let mut structure_ser = Vec::<u8>::with_capacity(len);
        for i in 0..len {
            structure_ser.push(self.read(4 + i));
        }

        postcard::from_bytes(structure_ser.deref()).ok()
    }

    #[doc(hidden)]
    fn get_structure_length(&self) -> u32 {
        let structure_length_bytes = self.get_structure_len_bytes();
        let structure_length = u32::from_ne_bytes(structure_length_bytes);
        return structure_length;
    }

    #[doc(hidden)]
    fn get_structure_len_bytes(&self) -> [u8; 4] {
        let mut structure_length_bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            structure_length_bytes[i] = self.read(i);
        }
        structure_length_bytes
    }

    #[doc(hidden)]
    fn get_structure_len_bytes_backup(&self) -> [u8; 4] {
        let mut structure_length_bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            structure_length_bytes[i] = self.read(BACKUP_ADDRESS + i);
        }
        structure_length_bytes
    }
}