
//
//  Kernel Main
//

#![allow(dead_code)]
#![feature(lang_items, unique, const_fn)]
#![no_std]

// A very basic crate that wraps a type so that the only way to access its
// contents is through volatile read/writes. Volatile read/writes are assumed by
// the compiler to have other side effects than just setting/getting a piece of
// memory, and thus the compiler does not optimise them out.
extern crate volatile;

// A very basic spin-lock mutex, used to wrap the static VGA buffer so that when
// separate threads attempt to write to the terminal, only one can do so at a
// time, preventing data races.
//
// A spin lock is the most basic form of mutex, which simply attempts to lock a
// mutex repeatedly in a while loop until it is successful.
extern crate spin;

// The Rust `core` crate has a number of basic dependencies like `memcpy`,
// `memset`, etc. In order to provide these symbols, we include a Rust
// implementation of them in an external library (instead of having to write
// them ourselves).
extern crate rlibc;

#[macro_use] mod drivers;
mod memory;
mod multiboot;

use drivers::vga;
use multiboot::{MultibootInfo, Header, MemoryAreaType};

use core::fmt::Arguments;

// This is the main Rust entry point for the kernel, called from the `start.asm`
// code after a bunch of configuration (like switching to long mode) is done.
//
// The assembly code calling this function passes a pointer to the multiboot
// information struct as the first argument.
#[no_mangle]
pub extern fn kernel_main(multiboot_ptr: usize) {
	// Initialisation
	vga::init();

	// Print all usable memory areas
	let info = unsafe { MultibootInfo::new(multiboot_ptr as *const Header) };
	println!("memory areas:");
	for area in info.memory_areas() {
		if area.kind() == MemoryAreaType::Usable {
			println!("  base {:#x}, size {:#x}", area.start(), area.size());
		}
	}

	// Print all kernel sections
	println!("kernel sections:");
	for section in info.sections() {
		println!("  base {:#x}, size {:#x}, flags {:#x}", section.start(), section.size(), section.flags);
	}

	// Don't return back to assembly
	loop {}
}

#[lang = "eh_personality"]
extern fn eh_personality() {
	// Do nothing for now
}

// This is called when a Rust function calls the `panic!` macro, and should
// print an error message and not return.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: Arguments, file: &'static str, line: u32) -> ! {
	// Print the file, line number, and provided format arguments with the panic
	println!("\n\npanic in file {} on line {}:", file, line);
	println!("  {}", fmt);

	// Make sure this function doesn't return (required by the ! return type)
	loop {}
}

// Although we disabled unwinding upon `panic!` calls in our kernel (so the
// compiler doesn't generate landing pads, which require a special gcc library),
// the `core` crate still has undefined references to `_Unwind_Resume`. To
// solve this, we just provide a dummy implementation for now.
//
// TODO: instead, recompile the `core` crate with unwinding disabled
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
