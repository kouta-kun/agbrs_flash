#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::Deref;

use agb::save::SaveData;
use serde::{Deserialize, Serialize};

pub struct FlashMemory {
    save_data: SaveData,
}


// needed bc otherwise it can't be pub static
unsafe impl Send for FlashMemory {}

unsafe impl Sync for FlashMemory {}

// store object length at the end of the cartridge to verify whether the cartridge has been written to and as a checksum
const BACKUP_ADDRESS: usize = 0xFFF0;

impl FlashMemory {
    /// Whether the cartridge Flash has a structure written to it
    pub fn have_structure(&mut self) -> bool {
        return self.verify_struct_len();
    }

    pub fn new_flash_128k(gba: &mut agb::Gba) -> Self {
        gba.save.init_flash_128k();
        let save_manager = gba.save.access().unwrap();
        Self { save_data: save_manager }
    }

    #[doc(hidden)]
    fn verify_struct_len(&mut self) -> bool {
        let len_bytes = self.get_structure_len_bytes();
        let len_bytes_backup = self.get_structure_len_bytes_backup();
        let len = u32::from_ne_bytes(len_bytes);
        return (len_bytes == len_bytes_backup) && len != 0xFFFFFFFF;
    }


    /// Persist a data structure to the cartridge (needs to be serde serializable)
    pub fn write_structure<T: Serialize>(&mut self, structure: &T) -> bool {
        let structure_ser = postcard::to_allocvec(structure).unwrap();
        agb::println!("{}", structure_ser.len());
        let structure_length: [u8; 4] = (structure_ser.len() as u32).to_ne_bytes();
        {
            let mut prepared_write_base = self.save_data.prepare_write(0..4 + structure_ser.len()).unwrap();
            if !prepared_write_base.write(0, structure_length.as_ref()).is_ok() { return false; }
            if !prepared_write_base.write(4, structure_ser.as_ref()).is_ok() { return false; }
        }
        let mut prepared_write_base = self.save_data.prepare_write(BACKUP_ADDRESS..BACKUP_ADDRESS + 4).unwrap();
        if !prepared_write_base.write(0, structure_length.as_ref()).is_ok() { return false; }
        return true;
    }

    /// Read a structure from cartridge flash (needs to be serde deserializable) or return a None if there isn't a structure
    pub fn read_structure<T: for<'a> Deserialize<'a>>(&mut self) -> Option<T> {
        let len = self.get_structure_length() as usize;
        let mut structure_ser = Vec::<u8>::with_capacity(len);
        structure_ser.resize(len, 0);

        self.save_data.read(4, structure_ser.as_mut()).unwrap();

        postcard::from_bytes(structure_ser.deref()).ok()
    }

    #[doc(hidden)]
    fn get_structure_length(&mut self) -> u32 {
        let structure_length_bytes = self.get_structure_len_bytes();
        let structure_length = u32::from_ne_bytes(structure_length_bytes);
        return structure_length;
    }

    #[doc(hidden)]
    fn get_structure_len_bytes(&mut self) -> [u8; 4] {
        let mut structure_length_bytes: [u8; 4] = [0; 4];
        self.save_data.read(0, &mut structure_length_bytes).unwrap();
        structure_length_bytes
    }

    #[doc(hidden)]
    fn get_structure_len_bytes_backup(&mut self) -> [u8; 4] {
        let mut structure_length_bytes: [u8; 4] = [0; 4];
        self.save_data.read(0, &mut structure_length_bytes).unwrap();
        structure_length_bytes
    }
}