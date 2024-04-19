from eth_keys import keys
import getpass  # Use getpass to hide the input for added security

def is_hex(s):
    try:
        int(s, 16)
        return True
    except ValueError:
        return False

def derive_public_key_from_private_key(private_key_hex):
    if not is_hex(private_key_hex) or len(private_key_hex) != 64:
        raise ValueError("Invalid private key. Please ensure it is a 64 hex characters string.")
    
    # Convert the private key hex string to a PrivateKey object
    private_key = keys.PrivateKey(bytes.fromhex(private_key_hex))
    
    # Derive the public key
    public_key = private_key.public_key
    
    # Return the public key in its hexadecimal representation
    return public_key.to_hex()

def main():
    try:
        # Prompt for private key securely
        private_key_hex = getpass.getpass('Enter your private key hex (input will be hidden): ')
        public_key_hex = derive_public_key_from_private_key(private_key_hex)
        print("Public Key:", public_key_hex)
    except ValueError as e:
        print(e)

if __name__ == "__main__":
    main()
