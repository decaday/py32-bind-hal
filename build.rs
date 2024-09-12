// use std::path::PathBuf;

// fn main() {
//     // #[cfg(feature = "auto_memory_x")]
//     // link_memory_x();
// }

// #[cfg(feature = "auto_memory_x")]
// fn link_memory_x(){
//     let memory_region_path = PathBuf::from("src/memory_region");

//     #[cfg(feature = "ram_rom_py32xxx6")]
//     let memory_x_path = memory_region_path.join("PY32xxx6");

//     #[cfg(feature = "ram_rom_py32xxx8")]
//     let memory_x_path = memory_region_path.join("PY32xxx8");

//     #[cfg(feature = "ram_rom_py32f002ax5")]
//     let memory_x_path = memory_region_path.join("PY32F002Ax5");


//     println!("cargo:rustc-link-search={}", memory_x_path.display());
// }
fn main() {}