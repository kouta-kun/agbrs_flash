#![no_main]
#![no_std]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};

extern crate alloc;

#[derive(Serialize, Deserialize)]
struct TestObject {
    pub id: usize,
    pub text: String,
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut memory = agbrs_flash::FlashMemory::new_flash_128k(&mut gba);
    let have_struct = memory.have_structure();
    agb::println!("Have structure? {}", have_struct);
    if !have_struct {
        memory.write_structure(&TestObject {
            id: 1,
            text: "Hello World!".to_string(),
        });
        agb::println!("Restart to see the stored data")
    } else {
        let obj: TestObject = memory.read_structure().unwrap();
        agb::println!("id: {} text: {}", obj.id, obj.text);
    }
    loop{}
}
