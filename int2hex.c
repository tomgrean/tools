#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static struct {
	const char *command;
	short from;
	short to;
}cmds[] = {
		{command:"int2bin", from:10, to:2 },
		{command:"int2oct", from:10, to:8 },
		{command:"int2hex", from:10, to:16},
		{command:"hex2bin", from:16, to:2 },
		{command:"hex2oct", from:16, to:8 },
		{command:"hex2int", from:16, to:10},
		{command:"oct2bin", from:8,  to:2 },
		{command:"oct2int", from:8,  to:10},
		{command:"oct2hex", from:8,  to:16},
		{command:"bin2oct", from:2,  to:8 },
		{command:"bin2int", from:2,  to:10},
		{command:"bin2hex", from:2,  to:16},

		{command:NULL, from:0, to:0}
};
const char digit[] = "0123456789abcdefghijklmnopqrstuvwxyz";

short getOptValue(const char *opt)
{
	for (; *opt; ++opt) {
		if (*opt != ' ' && *opt != '\t') {
			return (short)strtol(opt, NULL, 10);
		}
	}
	return -1;
}
void convertToAny(long value, const short to)
{
	if (value < 0) {
		printf("not impl");
		return;
	}
	if (value > 0) {
		convertToAny(value / to, to);
		printf("%c", digit[value % to]);
	}
}
int main(int argc, char *argv[])
{
	long value;
	int i;
	short from = -1, to;
	short fromk = 0, tok = 0;

	to = strlen(argv[0]);
	while (to > 0 && argv[0][to - 1] != '/') {
		--to;
	}
	for (i = 0; cmds[i].command; ++i) {
		if (!strcmp(cmds[i].command, argv[0] + to)) {
			from = cmds[i].from;
			to = cmds[i].to;
			break;
		}
	}
	for (i = 1; i < argc; ++i) {
		if (fromk) {
			from = getOptValue(argv[i]);
			fromk = 0;
			continue;
		} else if (tok) {
			to = getOptValue(argv[i]);
			tok = 0;
			continue;
		}
		if (argv[i][0] == '-') {
			if (argv[i][1] == 'f') {
				from = getOptValue(argv[i] + 2);
				if (from < 0) {
					fromk = 1;
				}
				continue;
			} else if (argv[i][1] == 't') {
				to = getOptValue(argv[i] + 2);
				if (to < 0) {
					tok = 1;
				}
				continue;
			}
			// no minus number for now.
			continue;
		}
		//real parameters.
		if (from < 0) {
			fprintf(stderr, "wrong 'from' radix!\n");
			return 2;
		}
		if (to < 0) {
			fprintf(stderr, "wrong 'to' radix!\n");
			return 2;
		}

		value = strtol(argv[i], NULL, from);
		switch (to) {
		case 8:
			printf("\t%lo", value);
			break;
		case 10:
			printf("\t%ld", value);
			break;
		case 16:
			printf("\t%lx", value);
			break;
		default:
			if (to < sizeof(digit)) {
				convertToAny(value, to);
			} else {
				fprintf(stderr, "radix:%d too big.\n", to);
			}
			break;
		}
	}
	printf("\n");
	return 0;
}
