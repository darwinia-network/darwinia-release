// std
use std::{env, error::Error, fs};
// crates.io
use blake2_rfc::blake2b;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let p = args.get(1).expect("missing WASM file path");
    let whitelist_pi = args
        .get(2)
        .expect("missing whitelist pallet index")
        .trim_start_matches("0x");
    let parachain_system_pi = args
        .get(3)
        .expect("missing parachain system pallet index")
        .trim_start_matches("0x");
    let w = fs::read(p)?;
    let h1 = blake2b::blake2b(32, &[], &w);
    let h1 = h1.as_bytes();
    let mut c = vec![
        u8::from_str_radix(whitelist_pi, 16)?,
        0x03,
        u8::from_str_radix(parachain_system_pi, 16)?,
        0x02,
    ];

    c.extend_from_slice(h1);
    c.push(0x01);

    let h2 = blake2b::blake2b(32, &[], &c);

    println!("{}", array_bytes::bytes2hex("0x", h2));

    Ok(())
}
