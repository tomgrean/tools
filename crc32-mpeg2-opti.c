/*
 * according to https://create.stephan-brumme.com/crc32/#git1
 * I managed to build a version used for CRC32-MPEG.
 * The code can only be used in little-endian systems.
 * using both slice-by-8 and slice-by-4.
 * I also managed to find a more generic version in Linux src.
 * People who want other version of CRC32 may find it useful.
 *
 * tomgrean at github dot com
 *
 * GPL(General Public License)
 */

#include <inttypes.h>
#include <byteswap.h>
#include <stdio.h>

#define Polynomial 0x04c11db7
#define MaxSlice 8
uint32_t Crc32Lookup[MaxSlice][256];

static void init()
{
	for (uint32_t i = 0; i <= 0xff; i++) {
		uint32_t crc32 = i << 24;
		for (int j = 0; j < 8; j++) {
			crc32 = (crc32 & 0x80000000) ? ((crc32 << 1) ^ Polynomial) : (crc32 << 1);
		}
		Crc32Lookup[0][i] = bswap_32(crc32);
	}
	for (int i = 0; i <= 0xff; i++) {
		// for Slicing-by-4 and Slicing-by-8
		Crc32Lookup[1][i] = (Crc32Lookup[0][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[0][i] & 0xff];
		Crc32Lookup[2][i] = (Crc32Lookup[1][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[1][i] & 0xff];
		Crc32Lookup[3][i] = (Crc32Lookup[2][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[2][i] & 0xff];
		// only Slicing-by-8
		Crc32Lookup[4][i] = (Crc32Lookup[3][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[3][i] & 0xff];
		Crc32Lookup[5][i] = (Crc32Lookup[4][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[4][i] & 0xff];
		Crc32Lookup[6][i] = (Crc32Lookup[5][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[5][i] & 0xff];
		Crc32Lookup[7][i] = (Crc32Lookup[6][i] >> 8) ^ Crc32Lookup[0][Crc32Lookup[6][i] & 0xff];
	}
}
uint32_t crc32_mpeg2_slice(const uint8_t *data, int len)
{
	uint32_t crc32 = 0xffffffff;
	const uint32_t *current = (const uint32_t*) data;
	--current;

	while (len >= 8) {
		uint32_t one = crc32 ^ *++current;
		uint32_t two = *++current;

		crc32 = Crc32Lookup[0][(two >> 24) & 0xff] ^
				Crc32Lookup[1][(two >> 16) & 0xff] ^
				Crc32Lookup[2][(two >> 8 ) & 0xff] ^
				Crc32Lookup[3][two         & 0xff] ^
				Crc32Lookup[4][(one >> 24) & 0xff] ^
				Crc32Lookup[5][(one >> 16) & 0xff] ^
				Crc32Lookup[6][(one >>  8) & 0xff] ^
				Crc32Lookup[7][one         & 0xff];
		len -= 8;
	}

	while (len >= 4) {
		uint32_t one = crc32 ^ *++current;
		crc32 = Crc32Lookup[0][(one >> 24) & 0xff] ^
				Crc32Lookup[1][(one >> 16) & 0xff] ^
				Crc32Lookup[2][(one >>  8) & 0xff] ^
				Crc32Lookup[3][one         & 0xff];
		len -= 4;
	}

	if (len) {
		data = (const uint8_t*) (current + 1) - 1;
		// remaining bytes (standard algorithm)
		do {
			crc32 = (crc32 >> 8) ^ Crc32Lookup[0][(crc32 ^ *++data) & 0xff];
		} while (--len > 0);
	}

	return bswap_32(crc32);
}









// test data
const static uint8_t demo[] = {
	0x97, 0x43, 0xa4, 0x18, 0xf0, 0x73, 0x30, 0x1a,
	0xdb, 0xdb, 0x08, 0x03, 0x25, 0xf0, 0x0f, 0x58,
	0x0d, 0x43, 0x48, 0x4e, 0x02, 0x08, 0x00, 0xe7,
	0x5a, 0xd0, 0xc0, 0x0e, 0x08, 0x08
};
// main demo code, and test.
int main()
{
	init();

	for (int m = 0; m < 8; ++m) {
	printf("{");
	for (int i = 0; i < 256; ++i) {
		if (i % 8 == 0) {
			printf("\n");
		}
		printf("0x%08x,", Crc32Lookup[m][i]);
	}
	printf("\n},");
	}
	printf("\n----\n");

	uint32_t result = crc32_mpeg2_slice((const uint8_t*)"123456789", 9);
	printf("%x\n===========================\n", result);
	result = crc32_mpeg2_slice(demo, sizeof(demo));
	printf("1=%x\n", result);
	return 0;
}

