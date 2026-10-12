/* Cross-implementation helper for tests/test_python.py.
 *   proto_tool encode   stdin lines "type seq hexpayload" -> stdout wire bytes as hex, one frame per line
 *   proto_tool decode   stdin raw bytes -> stdout "type seq hexpayload" per valid frame, then "# ok crc frame gaps" */
#include <stdio.h>
#include <string.h>
#include "sennep_proto.h"

static int hexval(char c) { return c >= '0' && c <= '9' ? c - '0' : c >= 'a' && c <= 'f' ? c - 'a' + 10 : -1; }

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "encode") == 0) {
        unsigned type, seq;
        char hex[2 * SP_MAX_PAYLOAD + 4];
        while (scanf("%u %u %130s", &type, &seq, hex) == 3) {
            uint8_t p[SP_MAX_PAYLOAD], wire[SP_MAX_WIRE];
            size_t n = strcmp(hex, "-") == 0 ? 0 : strlen(hex) / 2;
            for (size_t i = 0; i < n; i++) p[i] = (uint8_t)(hexval(hex[2 * i]) * 16 + hexval(hex[2 * i + 1]));
            size_t w = sp_frame_encode((uint8_t)type, (uint8_t)seq, p, (uint8_t)n, wire);
            for (size_t i = 0; i < w; i++) printf("%02x", wire[i]);
            printf("\n");
        }
        return 0;
    }
    if (argc == 2 && strcmp(argv[1], "decode") == 0) {
        sp_decoder_t d;
        sp_decoder_init(&d);
        sp_frame_t f;
        int c;
        while ((c = getchar()) != EOF) {
            if (sp_decoder_feed(&d, (uint8_t)c, &f)) {
                printf("%u %u ", f.type, f.seq);
                if (!f.len) printf("-");
                for (uint8_t i = 0; i < f.len; i++) printf("%02x", f.payload[i]);
                printf("\n");
            }
        }
        printf("# %u %u %u %u\n", d.frames_ok, d.crc_errors, d.frame_errors, d.seq_gaps);
        return 0;
    }
    fprintf(stderr, "usage: proto_tool encode|decode\n");
    return 2;
}
