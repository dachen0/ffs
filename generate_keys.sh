#!/bin/sh
openssl genpkey -algorithm ED25519 -out ffs_private_key.pem
openssl pkey -in ffs_private_key.pem -pubout > ffs_public_key.pem