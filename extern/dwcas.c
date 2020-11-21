#include <stdint.h>

struct dwcas_uint128_t {
  uint64_t first, second;
};

#ifdef _MSC_VER
#include <intrin.h>

uint8_t dwcas_compare_exchange_128(
  volatile struct dwcas_uint128_t* dst,
  struct dwcas_uint128_t*          old,
  struct dwcas_uint128_t           new,
  uint8_t                          success,
  uint8_t                          failure
) {
  (void) success;
  (void) failure;

  __int64* exchange = (__int64*) &new;
  return (uint8_t) __InterlockedCompareExchange128(
    (volatile __int64*) dst,
    exchange[0],
    exchange[1],
    (__int64*) old
  );
}
#else
static inline uint8_t dwcas_transform_memorder(uint8_t order) {
  switch (order) {
    case 0:  return __ATOMIC_RELAXED;
    case 2:  return __ATOMIC_ACQUIRE;
    case 3:  return __ATOMIC_RELEASE;
    case 4:  return __ATOMIC_ACQ_REL;
    default: return __ATOMIC_SEQ_CST;
  }
}

uint8_t dwcas_compare_exchange_128(
  volatile struct dwcas_uint128_t* dst,
  struct dwcas_uint128_t*          old,
  struct dwcas_uint128_t           new,
  uint8_t                          success,
  uint8_t                          failure
) {
  return (uint8_t) __atomic_compare_exchange_n(
    (volatile __uint128_t*) dst,
    (__uint128_t*) old,
    *((__uint128_t*) &new),
    0,
    dwcas_transform_memorder(success),
    dwcas_transform_memorder(failure)
  );
}
#endif