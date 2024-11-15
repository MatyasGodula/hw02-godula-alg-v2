// this code was written by chatgpt I AM NOT THE AUTHOR
#include <stdio.h>
#include <unistd.h>

int main() {
    // Path to the Rust binary
    char *rust_binary_path = "./runnable_binary";

    // Run the Rust binary using execl
    if (execl(rust_binary_path, rust_binary_path, NULL) == -1) {
        perror("Error running Rust binary");
        return 1;
    }

    return 0; // This line is never reached if execl is successful
}
