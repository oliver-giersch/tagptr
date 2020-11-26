#include <stdint.h>

uint8_t dwcas_compare_exchange128(
  volatile uint64_t dst[2],
  uint64_t          old[2],
  const uint64_t    new[2],
  uint8_t           success,
  uint8_t           failure
);

#ifdef _MSC_VER
#include <intrin.h>

uint8_t dwcas_compare_exchange128(
  volatile uint64_t dst[2],
  uint64_t          old[2],
  const uint64_t    new[2],
  uint8_t           success,
  uint8_t           failure
) {
  (void) success;
  (void) failure;

  const __int64* exchange = (const __int64*) new;
  return (uint8_t) _InterlockedCompareExchange128(
    (volatile __int64*) dst,
    exchange[1],
    exchange[0],
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

uint8_t dwcas_compare_exchange128(
  volatile uint64_t dst[2],
  uint64_t          old[2],
  const uint64_t    new[2],
  uint8_t           success,
  uint8_t           failure
) {
  return (uint8_t) __atomic_compare_exchange_n(
    (volatile __uint128_t*) dst,
    (__uint128_t*) old,
    *((const __uint128_t*) new),
    0,
    dwcas_transform_memorder(success),
    dwcas_transform_memorder(failure)
  );
}
#endif
