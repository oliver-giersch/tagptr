#ifndef CONQUER_POINTER_DWCAS_H
#define CONQUER_POINTER_DWCAS_H

#include <stdint.h>

typedef struct uint128_t {
  uint64_t first, second;
} uint128_t;

uint8_t dwcas_compare_exchange_128(
  uint128_t* dest,
  uint128_t old,
  uint128_t new,
);

#endif /* CONQUER_POINTER_DWCAS_H */
