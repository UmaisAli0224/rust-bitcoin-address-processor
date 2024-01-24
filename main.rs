use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use rust_cuda::prelude::*;
use bitcoin::network::constants::Network;
use bitcoin::util::bip32::{ExtendedPrivKey, KeySource, ChildNumber};
use bitcoin::util::address::Address;

const MAX_LINES_PER_FILE: usize = 1000000;
const CHUNK_SIZE: usize = 10000;

fn main() -> Result<(), Box<dyn Error>> {
    // Input and output folder paths
    let input_folder = r"C:\Users\Umais Ali\Desktop";
    let output_folder = r"C:\Users\Umais Ali\Desktop";

    // List all files in the input folder
    let files = std::fs::read_dir(input_folder)?;

    // Create CUDA context
    let ctx = Context::new()?;

    // Iterate through each file
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();

        // Check if the file has a .txt extension
        if file_name.ends_with(".txt") {
            let input_path = format!("{}/{}", input_folder, file_name);
            let output_base_name = file_name.trim_end_matches(".txt");

            process_file(&ctx, &input_path, output_folder, output_base_name)?;
        }
    }

    Ok(())
}

fn process_file(ctx: &Context, input_path: &str, output_folder: &str, output_base_name: &str) -> Result<(), Box<dyn Error>> {
    // Read input file
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    // Initialize buffers
    let mut seeds = Vec::new();
    let mut buffer = String::new();

    // Read seed phrases into a vector
    for line in reader.lines() {
        let line = line?;
        
        // Skip non-seed phrase lines
        if line.trim().is_empty() || !line.chars().all(|c| c.is_alphanumeric()) {
            continue;
        }

        buffer.push_str(&line);
        buffer.push('\n');

        if buffer.len() >= CHUNK_SIZE {
            seeds.push(buffer.clone());
            buffer.clear();
        }
    }

    // Process chunks in parallel
    let processed_data: Vec<String> = seeds.par_iter()
        .map(|chunk| process_chunk(ctx, chunk))
        .collect();

    // Write processed data to output files
    for (index, data) in processed_data.iter().enumerate() {
        let output_path = format!("{}/{}_part{}.txt", output_folder, output_base_name, index);
        let mut output_file = File::create(output_path)?;
        output_file.write_all(data.as_bytes())?;
    }

    Ok(())
}

fn process_chunk(ctx: &Context, chunk: &str) -> String {
    // Split chunk into lines
    let seeds: Vec<&str> = chunk.lines().collect();

    // Process each seed in parallel
    let processed_data: Vec<String> = seeds.par_iter()
        .map(|seed| derive_addresses(seed))
        .collect();

    // Join results into a single string
    processed_data.join("\n") + "\n"
}

fn derive_addresses(seed: &str) -> String {
    // Attempt to derive Bitcoin addresses using BIP44, BIP49, and BIP84
    match bitcoin::util::bip39::Mnemonic::from_str(seed) {
        Ok(mnemonic) => {
            let seed_data = mnemonic.to_seed("");
            let root = ExtendedPrivKey::new_master(Network::Bitcoin, &seed_data).unwrap();

            let addresses: Vec<String> = vec![
                root.derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(44).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .address(Network::Bitcoin).to_string(),

                root.derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(49).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .address(Network::Bitcoin).to_string(),

                root.derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(84).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_hardened_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .derive_pubkey(&ctx, &ChildNumber::from_normal_idx(0).unwrap())
                    .address(Network::Bitcoin).to_string(),
            ];

            addresses.join(" ")
        }
        Err(_) => format!("Error: Invalid seed phrase - {}", seed),
    }
}
