## FFS
A fast file sender.

# Design
UDP only. Only protocol packets are signed for handshake and metadata to reduce signing overhead. Data packets are sent raw and checked client side against received hashes.