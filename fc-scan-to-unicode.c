#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define CODE_PER_LINE 8
struct ScanCode {
	unsigned int index;
	unsigned int code[CODE_PER_LINE];
};

static struct ScanCode *psc;
static int scSize = 0;
static int scMalloc = 10;

static void initScanCode()
{
	psc = malloc(scMalloc * sizeof(struct ScanCode));
	if (!psc) {
		fprintf(stderr, "error malloc!\n");
		exit(2);
	}
}
static void ensureScanCode(int esize)
{
	//fprintf(stderr, "in ensureScanCode(%d)!\n", esize);
	if (scMalloc >= esize) {
		return;
	}
	scMalloc *= 2;
	struct ScanCode *np = realloc(psc, scMalloc * sizeof(struct ScanCode));
	if (!np) {
		fprintf(stderr, "error realloc(_, %d)!\n", scMalloc);
		exit(2);
	}
	psc = np;
	ensureScanCode(esize);
}
static void appendScanCode(struct ScanCode *a)
{
	memcpy(&psc[scSize], a, sizeof(struct ScanCode));
	++scSize;
}

static void output_unicode_16(unsigned int ucode)
{
	char p;

	//1Byte
	if (ucode < 0x80) {
		printf("%c", ucode);
		return;
	}
	//2Bytes
	if (ucode >= 0x80 && ucode <= 0x7ff) {
		printf("%c%c",
				(((ucode >> 6) & 0x1f) | 0xc0),
				((ucode & 0x3f) | 0x80));
		return;
	}
	//4Bytes(proxy...)
	if (ucode >= 0xd800 && ucode <= 0xdbff) {
		//unspecified...
		fprintf(stderr, "__zz__%x__ZZ__", ucode);
		return;
	}
	//3Bytes
	// * U-0000E000 - U-0000FFFF:  1110xxxx 10xxxxxx 10xxxxxx
	printf("%c%c%c",
			(char)(((ucode >> 12) & 0x0f) | 0xe0),
			(char)(((ucode >> 6) & 0x3f) | 0x80),
			(char)((ucode & 0x3f) | 0x80));
}
//read from standard input,
//write to standard output.
int main()
{
	struct ScanCode tmp;
	int lines = 0;
	initScanCode();
	while (1) {
		int ret;
		++lines;
		ret = scanf(" %x: %x %x %x %x %x %x %x %x",
				&tmp.index,
				&tmp.code[0],
				&tmp.code[1],
				&tmp.code[2],
				&tmp.code[3],
				&tmp.code[4],
				&tmp.code[5],
				&tmp.code[6],
				&tmp.code[7]);
		if (9 != ret) {
			break;
		}
		ensureScanCode(lines);
		appendScanCode(&tmp);
	}
	fprintf(stderr, "read %d lines.\n", lines);
	for (lines = 0; lines < scSize; ++lines) {
		unsigned int unicode = (32 * 8) * psc[lines].index;
		int col;
		for (col = 0; col < 8; ++col) {
			int bit;
			int i;
			for (i = 0, bit = 1; i < 32; bit <<= 1, ++i) {
				if (psc[lines].code[col] & bit) {
					output_unicode_16(unicode + i);
				}
			}
			unicode += 32;
		}
		printf("\n");//append a new line
	}


	free(psc);
	return 0;
}
