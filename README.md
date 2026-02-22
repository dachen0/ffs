# FFS
A fast file sender.

## Design
UDP only. Only protocol packets are signed for handshake and metadata to reduce signing overhead. Data packets are sent raw and checked client side against received hashes.

## Testing
Run the client with `cargo run -- --file-path test_out --mode client --ip 127.0.0.1 --udp-port 4200`

Then, run the server/sender with `cargo run -- --file-path test_in --mode server --ip 127.0.0.1 --udp-port 4201 --recipients 127.0.0.1:4200`

This will download the file at `test_in` to `test_out`.