#include <stdio.h>
#include <stdint.h>

extern uint8_t* alloc_buffer(size_t size);
extern void free_buffer(uint8_t* ptr, size_t size);

int main() {
    size_t size = 10;
    uint8_t* buf = alloc_buffer(size);

    for (size_t i = 0; i < size; ++i) {
        buf[i] = (uint8_t)(i + 1);
    }

    for (size_t i = 0; i < size; ++i) {
        printf("%02x ", buf[i]);
    }
    printf("\n");

    free_buffer(buf, size);
    return 0;
}
