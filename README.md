# FFS
A fast file sender.

## Design
UDP only. Only protocol packets are signed for handshake and metadata to reduce signing overhead. Data packets are sent raw and checked client side against received hashes.

## Usage
Generate the private key using `generate_keys.sh`. This will output the keys to `ffs_private_key.pem` and `ffs_public_key.pem`. DO NOT SHARE THE PRIVATE KEY. Copy the public key pem file to the client. This is so the client can verify that the file is being sent by the server.

Run the client with `cargo run -- --file-path test_out --mode client --ip 127.0.0.1 --udp-port 4200 --key-file-path ffs_public_key.pem`

Then, run the server/sender with `cargo run -- --file-path test_in --mode server --ip 127.0.0.1 --udp-port 4201 --recipients 127.0.0.1:4200 --key-file-path ffs_private_key.pem`

This will send the file at `test_in` to `test_out`.