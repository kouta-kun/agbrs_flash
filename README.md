# agbrs_flash

agbrs_flash is a crate that allows for persisting game data to a GBA cartridge flash chip. Currently it only supports 1M cartridges and it auto-identifies the resulting ROM as such.

It makes use of [postcard](https://crates.io/crates/postcard) to serialize the object and store it in the flash memory.

## Example

```rust
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
    agbrs_flash::MEMORY.init();
    let have_struct = agbrs_flash::MEMORY.have_structure();
    agb::println!("Have structure? {}", have_struct);
    if !have_struct {
        agbrs_flash::MEMORY.write_structure(&TestObject {
            id: 1,
            text: "Hello World!".to_string(),
        });
        agb::println!("Restart to see the stored data")
    } else {
        let obj: TestObject = agbrs_flash::MEMORY.read_structure().unwrap();
        agb::println!("id: {} text: {}", obj.id, obj.text);
    }
    loop{}
}
```