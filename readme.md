A simple RISC-V kernel

## Dev/Test Environment Setup
### Install the Nix package manager
If you already have the Nix package manager installed on your system then you can skip this step. \
The installation process might be [different](https://nixos.org/download/#nix-install-linux) for you distribution but on Ubuntu the following steps will suffice. 
```shell
# Install SSL certificates if not present
sudo apt-get update && sudo apt-get install -y curl xz-utils

# Download and run the Nix installer script
curl -L https://nixos.org/nix/install | sh

# Source nix profile (for current session)
. ~/.nix-profile/etc/profile.d/nix.sh
# or
source ~/.nix-profile/etc/profile.d/nix.sh

# Verify installation
nix --version
```

Remember to active the Nix shell by running  `$ nix-shell` in the project's root directory.

### Building the kernel
```shell
$ just kernel
```

### Running the kernel
```shell
$ just run
```

### Debugging
Run the kernel with the debug option
```shell
$ just run-gdb
```
And then on a different terminal you can attach to the debug session with GDB
```shell
$ just gdb
```

